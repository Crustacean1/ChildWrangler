use std::{
    cmp::{max, min},
    collections::{BTreeMap, HashMap},
    iter,
};

use chrono::{Datelike, Days, Month, Months, NaiveDate, Utc, Weekday};
use dto::attendance::{
    CateringMealDto, EffectiveAttendance, EffectiveMonthAttendance, GetEffectiveMonthAttendance,
    GetMonthAttendanceDto, MonthAttendanceDto,
};
use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, logging::log, prelude::*};

use leptos_router::{components::A, hooks::use_params};
use uuid::Uuid;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::Array,
    wasm_bindgen::{prelude::Closure, JsValue},
    window, Blob, FileSystemFileHandle, FileSystemWritableFileStream,
};

use crate::{
    components::{
        loader::Loader,
        modal::Modal,
        modals::{
            meal_count_modal::MealCountModal, meal_edit_modal::MealEditModal,
            meal_history_modal::MealHistoryModal,
        },
        snackbar::{self, use_snackbar, SnackbarContext},
    },
    icons::{
        download::DownloadIcon, left_arrow::LeftArrow, right_arrow::RightArrow, select::SelectIcon,
    },
    pages::attendance_page::{AttendanceParams, GroupVersion},
    services::attendance::{get_effective_attendance, get_month_attendance, get_monthly_summary},
};

#[component]
pub fn Calendar() -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();

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
            )
        },
        |(year, month, target, _)| async move {
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
            )
        },
        |(year, month, target, _)| async move {
            get_effective_attendance(GetEffectiveMonthAttendance {
                year: year as i32,
                month,
                target,
            })
            .await
        },
    );

    view! {
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
    OtherDow(NaiveDate),
    Day(NaiveDate, Vec<(Uuid, String, u32, EffectiveAttendance)>),
}

#[component]
pub fn InnerCalendar(
    target: Uuid,
    year: i32,
    month: u32,
    attendance: MonthAttendanceDto,
    local_attendance: EffectiveMonthAttendance,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let next_month =
        NaiveDate::from_ymd_opt(year, month, 1).and_then(|d| d.checked_add_months(Months::new(1)));
    let prev_month =
        NaiveDate::from_ymd_opt(year, month, 1).and_then(|d| d.checked_sub_months(Months::new(1)));

    let (meal_history, set_meal_history) = signal(None::<(Uuid, Uuid, NaiveDate)>);
    let (meal_count, set_meal_count) = signal(None::<(Uuid, Uuid, NaiveDate)>);
    //let (meal_count, set_meal_count) = signal(false);
    let (meal_edit, set_meal_edit) = signal(None::<Vec<_>>);
    let (selection_mode, set_selection_mode) = signal(false);

    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let (drag_start, set_drag_start) = signal(None::<NaiveDate>);
    let (drag_end, set_drag_end) = signal(None::<NaiveDate>);

    let selection_filter = |mode: bool, start: NaiveDate, end: NaiveDate, day: NaiveDate| {
        return false;
    };

    let download_summary = {
        Action::new(move |_: &()| async move {
            if let Ok(summary) = get_monthly_summary(target, year, month).await {
                save_to_file(&summary).await;
            } else {
                snackbar.error("Nie udało się pobrac danych o obecności", "");
            }
        })
    };

    let on_download = move |_| {
        download_summary.dispatch(());
    };

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

    let is_active = move |mode,
                          r_start: NaiveDate,
                          r_end: NaiveDate,
                          day: NaiveDate,
                          dow: &[bool],
                          att_start: &NaiveDate,
                          att_end: &NaiveDate| {
        let start = if r_start < r_end { r_start } else { r_end };
        let end = if r_start < r_end { r_end } else { r_start };

        if *att_start <= day
            && day <= *att_end
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

    let dow1 = attendance.days_of_week.clone();
    let on_drag_end = move || {
        if let (Some(r_start), Some(r_end)) = (drag_start(), drag_end()) {
            let start = min(r_start, r_end);
            let end = max(r_start, r_end);
            let mode = selection_mode();

            let days = iter::successors(NaiveDate::from_ymd_opt(year, month, 1), |day| {
                if day.month() == month {
                    day.checked_add_days(Days::new(1))
                } else {
                    None
                }
            })
            .filter(|d| {
                is_active(
                    mode,
                    start,
                    end,
                    *d,
                    &dow1,
                    &attendance.start,
                    &attendance.end,
                )
            });
            let days: Vec<_> = days.collect();
            set_meal_edit(Some(days));
        }
        set_drag_start(None);
        set_drag_end(None);
    };

    let daily_attendance = calendar_days.map(|day| {
        if day.month() != month {
            CalendarDay::OtherMonth
        } else if !attendance.days_of_week[day.weekday().num_days_from_monday() as usize]
            || day < attendance.start
            || day > attendance.end
        {
            CalendarDay::OtherDow(day)
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
            CalendarDay::Day(day, meals)
        }
    });

    let change_month = |date: Option<NaiveDate>| {
        move || {
            date.map(|next| format!("/attendance/{}/{}/{}", target, next.year(), next.month()))
                .unwrap_or(format!("/attendance/{}", target))
        }
    };
    let meals2 = attendance.meals.clone();

    view! {
        <div class="vertical gap flex-1 flex" on:mouseup=move |_| on_drag_end()>
            <div class="background-2 rounded padded gap horizontal center">
                <div class="flex-1"></div>
                <div class="flex-1 horizontal gap align-center space-between">
                    <A href=change_month(prev_month)>
                        <span class="icon-button interactive" title="Poprzedni miesiąc">
                            <LeftArrow />
                        </span>
                    </A>
                    <h3 class="h3" style:width="10em">
                        {move || {
                            NaiveDate::from_ymd_opt(year, month, 1)
                                .map(|d| format!("{}", d.format("%Y %B")))
                                .unwrap_or(String::new())
                        }}
                    </h3>
                    <A href=change_month(next_month)>
                        <span class="icon-button interactive" title="Następny miesiąc">
                            <RightArrow />
                        </span>
                    </A>
                </div>
                <div class="flex-1 flex-end horizontal gap">
                    <button
                        class="icon-button interactive"
                        title="Pobierz obecność"
                        on:click=on_download
                    >
                        <DownloadIcon />
                    </button>
                    <button
                        class="icon-button interactive"
                        class:outline-white=selection_mode
                        title="Przełącz tryb zaznaczania"
                        on:click=move |_| set_selection_mode(!selection_mode())
                    >
                        <SelectIcon />
                    </button>
                </div>
            </div>

            <div class="background-2 flex-1 rounded padded grid-7">
                {iter::successors(
                        Some(Weekday::Mon),
                        |w| { if *w == Weekday::Sun { None } else { Some(w.succ()) } },
                    )
                    .map(|w| {
                        view! {
                            <div class="background-3 rounded padded no-select center">
                                {format!("{}", w)}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
                {daily_attendance
                    .into_iter()
                    .map(|day|
                    view!{
                        <div class="vertical gap background-3 rounded padded no-select outline-1-hover fast-transition"
            on:mousedown=move |_| set_drag_start(None)
            on:mouseup=move |_| set_drag_end(None)
            on:mouseover=move |_| set_drag_end(None)>
                        {
                    match day {
                        CalendarDay::OtherMonth => {
                            Either::Left(
                                Either::Left(
                                    view! {},
                                ),
                            )
                        }
                        CalendarDay::OtherDow(date) => {
                            Either::Left(
                                Either::Right(
                                    view! {
                                            <h3 class="h3 gray">
                                                {format!("{}", date.format("%d %B"))}
                                            </h3>
                                    },
                                ),
                            )
                        }
                        CalendarDay::Day(date, meals) => {
                            Either::Right(
                                view! {
                                    <Day
                                        date
                                        meals
                                        on_meal_select=move |meal_id| {
                                            set_meal_history(Some((meal_id, target, date)))
                                        }
                                        on_count_select=move |meal_id| {
                                            set_meal_count(Some((meal_id, target, date)))
                                        }
                                    />
                                },
                            )
                        }
                        }
                    }
                        </div>

                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>

        <Modal is_open=move || meal_history().is_some() on_close=move || set_meal_history(None)>
            {move || {
                meal_history()
                    .map(|(meal_id, target, date)| {
                        view! { <MealHistoryModal meal_id target date /> }
                    })
            }}
        </Modal>
        <Modal is_open=move || meal_count().is_some() on_close=move || set_meal_count(None)>
            {move || {
                meal_count()
                    .map(|(meal_id, target, date)| view! { <MealCountModal meal_id target date /> })
            }}
        </Modal>
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
                                        on_close=move |_| set_meal_edit(None)
                                    />
                                }
                            }
                        })
                }
            }
        </Modal>
    }
}

#[component]
pub fn Day(
    date: NaiveDate,
    meals: Vec<(Uuid, String, u32, EffectiveAttendance)>,
    on_meal_select: impl Fn(Uuid) + Send + Sync + Copy + 'static,
    on_count_select: impl Fn(Uuid) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    view! {
            <h3 class=" h3">{format!("{}", date.format("%e %B"))}</h3>
            {meals
                .into_iter()
                .map(|(meal_id, meal_name, attendance, status)| {
                    view! {
                        <div class="horizontal gap flex-1 horizontal align-center">
                            <div
                                class="flex-4 interactive padded rounded no-select text-left flex justify-center align-center"
                                on:click=move |_| on_meal_select(meal_id)
                                class:green=status == EffectiveAttendance::Present
                                class:red=status == EffectiveAttendance::Absent
                                class:yellow=status == EffectiveAttendance::Cancelled
                                class:gray=status == EffectiveAttendance::Blocked
                                on:mousedown=|e| {
                                    e.stop_propagation();
                                }
                            >
                                {meal_name.clone()}
                            </div>

                            <button
                                class="flex-1 padded no-select interactive rounded"
                                on:click=move |_| on_count_select(meal_id)
                                on:mousedown=|e| {
                                    e.stop_propagation();
                                }
                            >
                                {format!("{}", attendance)}
                            </button>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
    }
}
