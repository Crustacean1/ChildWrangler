use std::iter;

use chrono::{Datelike, Days, Months, NaiveDate, Utc, Weekday};
use dto::attendance::{
    EffectiveAttendance, EffectiveMonthAttendance, GetEffectiveMonthAttendance,
    GetMonthAttendanceDto, MonthAttendanceDto,
};
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, prelude::*};

use leptos::logging::log;
use leptos_router::{components::A, hooks::use_params};
use uuid::Uuid;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::Array, wasm_bindgen::JsValue, Blob, FileSystemFileHandle, FileSystemWritableFileStream,
};

use crate::{
    components::{
        loader::Loader,
        modal::Modal,
        modals::{meal_count_modal::MealCountModal, meal_edit_modal::MealEditModal},
        snackbar::{use_snackbar, SnackbarContext},
    },
    icons::{
        download::DownloadIcon, left_arrow::LeftArrow, right_arrow::RightArrow, select::SelectIcon,
    },
    pages::attendance_page::{AttendanceParams, AttendanceVersion, GroupVersion},
    services::attendance::{get_effective_attendance, get_month_attendance, get_monthly_summary},
};

#[component]
pub fn Calendar() -> impl IntoView {
    let GroupVersion(group_version, _) = use_context().unwrap();
    let AttendanceVersion(attendance_version, _) = use_context().unwrap();

    let params = use_params::<AttendanceParams>();
    let params = move || params.read();

    let year = move || {
        params()
            .as_ref()
            .ok()
            .and_then(|attendance| attendance.year)
            .unwrap_or(Utc::now().year() as u32)
    };

    let month = move || {
        params()
            .as_ref()
            .ok()
            .and_then(|attendance| attendance.month)
            .unwrap_or(Utc::now().month())
    };

    let target = move || {
        params()
            .as_ref()
            .ok()
            .and_then(|attendance| attendance.target)
    };

    let attendance = Resource::new(
        move || {
            (
                year(),
                month(),
                target().unwrap_or_default(),
                group_version(),
                attendance_version(),
            )
        },
        |(year, month, target, _, _)| async move {
            get_month_attendance(GetMonthAttendanceDto {
                target,
                year,
                month,
            })
            .await
        },
    );

    let local_attendance = Resource::new(
        move || {
            (
                year(),
                month(),
                target().unwrap_or_default(),
                group_version(),
                attendance_version(),
            )
        },
        |(year, month, target, _, _)| async move {
            get_effective_attendance(GetEffectiveMonthAttendance {
                year: year as i32,
                month,
                target,
            })
            .await
        },
    );

    let next_month = move || {
        NaiveDate::from_ymd_opt(year() as i32, month(), 1)
            .and_then(|d| d.checked_add_months(Months::new(1)))
    };
    let prev_month = move || {
        NaiveDate::from_ymd_opt(year() as i32, month(), 1)
            .and_then(|d| d.checked_sub_months(Months::new(1)))
    };

    let prev_month_href = move || {
        if let (Some(id), Some(prev)) = (target(), prev_month()) {
            format!("/attendance/{}/{}/{}", id, prev.year(), prev.month())
        } else {
            String::new()
        }
    };

    let next_month_href = move || {
        if let (Some(id), Some(next)) = (target(), next_month()) {
            format!("/attendance/{}/{}/{}", id, next.year(), next.month())
        } else {
            String::new()
        }
    };

    view! {
            <div class="bg-gray-900 rounded-xl outline outline-white/15 flex flex-row gap-2 p-2 select-none">
                <div class="flex-1"></div>
                <div class="flex-1 flex flex-row gap items-center place-content-between">
                    <A href=prev_month_href>
                        <span class="hover:bg-gray-700 cursor-pointer rounded-md" title="Poprzedni miesiąc">
                            <LeftArrow />
                        </span>
                    </A>
                    <h3 class="min-w-10 text-center">
                        {move || {
                            NaiveDate::from_ymd_opt(year() as i32, month(), 1)
                                .map(|d| format!("{}", d.format("%Y %B")))
                                .unwrap_or(String::new())
                        }}
                    </h3>
                    <A href=next_month_href>
                        <span class="hover:bg-gray-700 cursor-pointer rounded-md" title="Następny miesiąc">
                            <RightArrow />
                        </span>
                    </A>
                </div>
                <div class="flex-1 justify-end flex flex-row gap-2">
                    <button
                        class="hover:bg-gray-700 cursor-pointer p-1 rounded-md"
                        title="Pobierz obecność"
                    >
                        <DownloadIcon />
                    </button>
                    <button
                        class="hover:bg-gray-700 cursor-pointer p-1 rounded-md"
                        title="Przełącz tryb zaznaczania"
                    >
                        <SelectIcon />
                    </button>
                </div>
        </div>
        <Loader>
            {move || Suspend::new(async move {
                let attendance = attendance.await?;
                let local_attendance = local_attendance.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(
                    view! {

                        <InnerCalendar
                            target=target().unwrap_or_default()
                            year=year() as i32
                            month=month()
                            attendance
                            local_attendance
                        />
                    },
                )
            })}
        </Loader>
    }
}

async fn save_to_file(summary: &str) {
    let snackbar = use_snackbar();

    let array = Array::new();
    array.push(&JsValue::from_str(summary));

    if let Ok(blob) = Blob::new_with_str_sequence(&array) {
        wasm_bindgen_futures::spawn_local(async move {
            match async move {
                let promise = web_sys::window().map(|window| window.show_save_file_picker());
                if let Some(Ok(promise)) = promise {
                    let handle = JsFuture::from(promise)
                        .await
                        .and_then(|handle| handle.dyn_into::<FileSystemFileHandle>())?;
                    let writable = JsFuture::from(handle.create_writable())
                        .await
                        .and_then(|writable| writable.dyn_into::<FileSystemWritableFileStream>())?;
                    JsFuture::from(writable.write_with_blob(&blob)?).await?;
                    JsFuture::from(writable.close()).await?;
                }
                Ok::<_, JsValue>(())
            }
            .await
            {
                Ok(_) => snackbar.success("Zapisano obecność"),
                Err(_) => snackbar.error("Nie udało się zapisać obecności", ""),
            }
        });
    }
}

pub enum CalendarDay {
    OtherMonth,
    OtherDow,
    Day(Vec<(Uuid, String, u32, EffectiveAttendance)>),
}

#[component]
pub fn InnerCalendar(
    target: Uuid,
    year: i32,
    month: u32,
    attendance: MonthAttendanceDto,
    local_attendance: EffectiveMonthAttendance,
) -> impl IntoView {
    let AttendanceVersion(_, set_attendance_version) = use_context().unwrap();

    let snackbar = use_snackbar();
    let is_student = local_attendance.is_student;

    let (meal_history, set_meal_history) = signal(None::<(Uuid, Uuid, NaiveDate)>);
    let (meal_count, set_meal_count) = signal(None::<(Uuid, Uuid, NaiveDate)>);
    let (meal_edit, set_meal_edit) = signal(None::<Vec<_>>);
    let (selection_mode, set_selection_mode) = signal(false);

    let (drag_start, set_drag_start) = signal(None::<NaiveDate>);
    let (drag_end, set_drag_end) = signal(None::<NaiveDate>);

    let download_summary = {
        Action::new(move |_: &()| async move {
            if let Ok(summary) = get_monthly_summary(target, year, month).await {
                save_to_file(&summary).await;
            } else {
                snackbar.error("Nie udało się pobrac danych o obecności", "");
            }
        })
    };

    /*let on_download = move |_| {
        download_summary.dispatch(());
    };*/

    let end_date = NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|d| d.checked_add_months(Months::new(1)))
        .and_then(|d| {
            d.checked_add_days(Days::new(
                (7 - d.weekday().num_days_from_monday()) as u64 % 7,
            ))
        });

    let calendar_days = iter::successors(
        NaiveDate::from_ymd_opt(year, month, 1).and_then(|day| {
            day.checked_sub_days(Days::new(day.weekday().num_days_from_monday() as u64))
        }),
        |day| {
            end_date.as_ref().and_then(|end_date| {
                day.checked_add_days(Days::new(1)).and_then(|d| {
                    if d >= *end_date {
                        None
                    } else {
                        Some(d)
                    }
                })
            })
        },
    );

    let mut dow: [bool; 7] = [false; 7];
    for (i, present) in attendance.days_of_week.iter().enumerate() {
        dow[i] = *present;
    }

    let is_selected = move |day: NaiveDate| {
        let Some(r_start) = drag_start() else {
            return false;
        };
        let Some(r_end) = drag_end() else {
            return false;
        };
        let start = if r_start < r_end { r_start } else { r_end };
        let end = if r_start < r_end { r_end } else { r_start };
        let mode = selection_mode();

        if mode {
            let (start_week, end_week) = (start.iso_week().week(), end.iso_week().week());
            let day_week = day.iso_week().week();
            let (start_dow, end_dow) = (
                start.weekday().num_days_from_monday(),
                end.weekday().num_days_from_monday(),
            );
            let day_dow = day.weekday().num_days_from_monday();
            (start_week <= day_week && day_week <= end_week)
                && (start_dow <= day_dow && day_dow <= end_dow)
        } else {
            start <= day && day <= end
        }
    };

    let is_active = move |day: NaiveDate| {
        let Some(r_start) = drag_start() else {
            return false;
        };
        let Some(r_end) = drag_end() else {
            return false;
        };
        let att_start = attendance.start;
        let att_end = attendance.end;
        let start = if r_start < r_end { r_start } else { r_end };
        let end = if r_start < r_end { r_end } else { r_start };
        let mode = selection_mode();

        if att_start <= day
            && day <= att_end
            && dow
                .get(day.weekday().num_days_from_monday() as usize)
                .map(|b| *b)
                .unwrap_or(false)
        {
            if mode {
                let (start_week, end_week) = (start.iso_week().week(), end.iso_week().week());
                let day_week = day.iso_week().week();
                let (start_dow, end_dow) = (
                    start.weekday().num_days_from_monday(),
                    end.weekday().num_days_from_monday(),
                );
                let day_dow = day.weekday().num_days_from_monday();
                (start_week <= day_week && day_week <= end_week)
                    && (start_dow <= day_dow && day_dow <= end_dow)
            } else {
                start <= day && day <= end
            }
        } else {
            false
        }
    };

    let on_drag_end = move || {
        if let (Some(r_start), Some(r_end)) = (drag_start(), drag_end()) {
            let days = iter::successors(NaiveDate::from_ymd_opt(year, month, 1), |day| {
                if day.month() == month {
                    day.checked_add_days(Days::new(1))
                } else {
                    None
                }
            })
            .filter(|d| is_active(*d));
            let days: Vec<_> = days.collect();
            set_meal_edit(Some(days));
        }
        set_drag_start(None);
        set_drag_end(None);
    };

    let daily_attendance = calendar_days.map(|day| {
        if day.month() != month {
            (day, CalendarDay::OtherMonth)
        } else if !attendance.days_of_week[day.weekday().num_days_from_monday() as usize]
            || day < attendance.start
            || day > attendance.end
        {
            (day, CalendarDay::OtherDow)
        } else {
            let meals = attendance
                .meals
                .iter()
                .map(|m| {
                    (
                        m.id,
                        m.name.clone(),
                        *attendance
                            .attendance
                            .get(&day)
                            .and_then(|d| d.get(&m.id))
                            .unwrap_or(&0),
                        *local_attendance
                            .attendance
                            .get(&day)
                            .and_then(|a| a.get(&m.id))
                            .unwrap_or(&EffectiveAttendance::Present),
                    )
                })
                .collect::<Vec<_>>();
            (day, CalendarDay::Day(meals))
        }
    });

    view! {
            <div class="grid gap-2 grid-cols-7 overflow-auto p-0.5 select-none">
                {iter::successors(
                        Some(Weekday::Mon),
                        |w| { if *w == Weekday::Sun { None } else { Some(w.succ()) } },
                    )
                    .map(|w| {
                        view! {
                            <div class="p-2 text-center items-center justify-center row-span-auto">
                                {format!("{}", w)}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
         </div>
            <div class="flex-1 grid gap-2 grid-cols-7 overflow-auto p-0.5 select-none">
                {daily_attendance
                    .into_iter()
                    .map(|(date, calendar_day)| {
                        view! {
                            <div
                                class="flex flex-col overflow-hidden gap-1 align-center rounded-lg p-2 outline outline-white/15 hover:bg-gray-800 active:bg-gray-700"
                                class:bg-gray-800=move || is_selected(date)
                                class:bg-gray-900=move || !is_selected(date)
                                on:mousedown=move |_| set_drag_start(Some(date))
                                on:mouseup=move |_| on_drag_end()
                                on:mouseover=move |_| set_drag_end(Some(date))
                            >
                                {match calendar_day {
                                    CalendarDay::OtherMonth => Either::Left(Either::Left(view! {})),
                                    CalendarDay::OtherDow => {
                                        Either::Left(
                                            Either::Right(
                                                view! {
                                                    <h3 class="text-center text-gray-600">
                                                        {format!("{}", date.format("%d %B"))}
                                                    </h3>
                                                },
                                            ),
                                        )
                                    }
                                    CalendarDay::Day(meals) => {
                                        Either::Right(
                                            view! {
                                                <Day
                                                    date
                                                    is_student
                                                    meals
                                                    meal_select=meal_history
                                                    count_select=meal_count
                                                    on_unselect=move || {
                                                        set_meal_history(None);
                                                        set_meal_count(None);
                                                    }
                                                    on_meal_select=move |meal_id| {
                                                        set_meal_history(Some((meal_id, target, date)))
                                                    }
                                                    on_count_select=move |meal_id| {
                                                    }
                                                />
                                            },
                                        )
                                    }
                                }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

        <Modal is_open=move || meal_edit().is_some() on_close=move || set_meal_edit(None)>
            {
                let meals = attendance.meals.clone();
                move || {
                    meal_edit()
                        .map({
                            let meals = meals.clone();
                            move |days| {
                                view! {
                                    <MealEditModal
                                        target
                                        days
                                        meals
                                        on_close=move |changed| {
                                            log!("Helloł: {:?}", changed);
                                            if changed {
                                                log!("Hello");
                                                *set_attendance_version.write() += 1;
                                            }
                                            set_meal_edit(None)
                                        }
                                    />
                                }
                            }
                        })
                }
            }
        </Modal>

        {}
        {move || {
            meal_count()
                .map(|(meal_id, target, date)| {
                    view! {
                        <div class="calendar-meal-tooltip pretty-background">
                            <MealCountModal target meal_id date />
                        </div>
                    }
                })
        }}
    }
}

#[component]
pub fn Day(
    date: NaiveDate,
    is_student: bool,
    meals: Vec<(Uuid, String, u32, EffectiveAttendance)>,
    meal_select: impl Fn() -> Option<(Uuid, Uuid, NaiveDate)> + Send + Sync + Copy + 'static,
    count_select: impl Fn() -> Option<(Uuid, Uuid, NaiveDate)> + Send + Sync + Copy + 'static,
    on_meal_select: impl Fn(Uuid) + Send + Sync + Copy + 'static,
    on_count_select: impl Fn(Uuid) + Send + Sync + Copy + 'static,
    on_unselect: impl Fn() + Send + Sync + Copy + 'static,
) -> impl IntoView {
    view! {
        <h3 class="text-center flex-1 justify-start">{format!("{}", date.format("%e %B"))}</h3>
        {meals
            .into_iter()
            .map(|(meal_id, meal_name, attendance, status)| {
                view! {
                    <div class="flex-1 flex flex-row align-center">
                        <div
                            class="flex-4 padded no-select text-left flex justify-left align-center"
                            on:mouseover=move |_| on_meal_select(meal_id)
                            on:mouseleave=move |_| on_unselect()
                            class:calendar-anchor=move || {
                                if let Some((selected_meal_id, _, selected_date)) = meal_select() {
                                    return selected_meal_id == meal_id && selected_date == date
                                } else {
                                    false
                                }
                            }
                            class:green=status == EffectiveAttendance::Present
                            class:red=status == EffectiveAttendance::Absent
                            class:yellow=status == EffectiveAttendance::Cancelled
                            class:gray=status == EffectiveAttendance::Blocked
                        >
                            {meal_name.clone()}
                        </div>
                        {if !is_student {
                            Either::Left(
                                view! {
                                    <div
                                        class="flex-1 padded no-select rounded"
                                        class:calendar-anchor=move || {
                                            if let Some((selected_meal_id, _, selected_date)) = count_select() {
                                                return selected_meal_id == meal_id && selected_date == date
                                            } else {
                                                false
                                            }
                                        }
                                        on:mouseover=move |_| on_count_select(meal_id)
                                        on:mouseleave=move |_| on_unselect()
                                    >
                                        {format!("{}", attendance)}
                                    </div>
                                },
                            )
                        } else {
                            Either::Right(view! {})
                        }}
                    </div>
                }
            })
            .collect::<Vec<_>>()}
    }
}
