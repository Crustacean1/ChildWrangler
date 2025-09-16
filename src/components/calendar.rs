use std::{
    cmp::{max, min},
    iter,
};

use chrono::{Datelike, Days, Month, Months, NaiveDate, Utc, Weekday};
use leptos::{either::Either, logging::log, prelude::*};
use leptos_router::hooks::use_params;
use uuid::Uuid;

use crate::{
    components::{
        modal::Modal,
        modals::{meal_count_modal::MealCountModal, meal_edit_modal::MealEditModal},
    },
    dtos::attendance::{
        EffectiveAttendance, EffectiveMonthAttendance, GetEffectiveMonthAttendance,
        GetMonthAttendanceDto, MonthAttendanceDto,
    },
    icons::{
        download::DownloadIcon, left_arrow::LeftArrow, right_arrow::RightArrow, select::SelectIcon,
    },
    pages::attendance_page::AttendanceParams,
    services::attendance::{get_effective_attendance, get_month_attendance},
};

#[component]
pub fn Calendar() -> impl IntoView {
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
        move || (year(), month(), target().unwrap_or_default()),
        |(year, month, target)| async move {
            get_month_attendance(GetMonthAttendanceDto {
                target,
                year,
                month,
            })
            .await
        },
    );

    let local_attendance = Resource::new(
        move || (year(), month(), target().unwrap_or_default()),
        |(year, month, target)| async move {
            get_effective_attendance(GetEffectiveMonthAttendance {
                year: year as i32,
                month,
                target,
            })
            .await
        },
    );

    view! {
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! {}
            }>
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
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn InnerCalendar(
    target: Uuid,
    year: i32,
    month: u32,
    attendance: MonthAttendanceDto,
    local_attendance: EffectiveMonthAttendance,
) -> impl IntoView {
    //let (meal_history, set_meal_history) = signal(false);
    //let (meal_count, set_meal_count) = signal(false);
    let (meal_edit, set_meal_edit) = signal(None::<Vec<_>>);
    let (selection_mode, set_selection_mode) = signal(false);

    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let end = start
        .checked_add_months(Months::new(1))
        .and_then(|end| end.checked_sub_days(Days::new(1)))
        .unwrap();

    let (drag_start, set_drag_start) = signal(None::<NaiveDate>);
    let (drag_end, set_drag_end) = signal(None::<NaiveDate>);

    let dow = attendance.days_of_week.clone();
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
    let on_drag_end = {
        move |_| {
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
                log!("Detected: {:?} days", days.len());
                set_meal_edit(Some(days));
            }
            set_drag_start(None);
            set_drag_end(None);
        }
    };

    view! {
        <div class="vertical gap flex-1 flex" on:mouseup=on_drag_end>
            <div class="background-2 rounded padded gap horizontal center">
                <div class="flex-1"></div>
                <div class="flex-1 horizontal gap align-center space-between">
                    <button class="icon-button interactive" title="Poprzedni miesiąc">
                        <LeftArrow />
                    </button>
                    {move || {
                        NaiveDate::from_ymd_opt(year, month, 1)
                            .map(|d| format!("{}", d.format("%Y %B")))
                            .unwrap_or(String::new())
                    }}
                    <button class="icon-button interactive" title="Następny miesiąc">
                        <RightArrow />
                    </button>
                </div>
                <div class="flex-1 flex-end horizontal gap">
                    <button class="icon-button interactive" title="Pobierz obecność">
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
                {NaiveDate::from_ymd_opt(year, month, 1)
                    .map(|day| {
                        (0..day.weekday().num_days_from_monday())
                            .map(|_| view! { <div></div> })
                            .collect::<Vec<_>>()
                    })}
                {iter::successors(
                        Some(start),
                        |day| { if *day < end { day.checked_add_days(Days::new(1)) } else { None } },
                    )
                    .map(|day| {
                        view! {
                            <div
                                class="background-3 padded rounded vertical"
                                on:mousedown=move |_| {
                                    set_drag_start(Some(day));
                                    set_drag_end(Some(day));
                                }
                                on:mouseover=move |_| {
                                    set_drag_end(Some(day));
                                }
                                class:highlight={
                                    let dow = attendance.days_of_week.clone();
                                    move || match (drag_start(), drag_end()) {
                                        (Some(start), Some(end)) => {
                                            is_active(
                                                selection_mode(),
                                                start,
                                                end,
                                                day,
                                                &dow,
                                                &attendance.start,
                                                &attendance.end,
                                            )
                                        }
                                        _ => false,
                                    }
                                }
                            >
                                <span class="no-select padded rounded">
                                    {format!("{}", day.format("%d %B"))}
                                </span>
                                {if attendance
                                    .days_of_week[day.weekday().num_days_from_monday() as usize]
                                    && day >= attendance.start && day <= attendance.end
                                {
                                    Either::Left(
                                        // Nested too deep: TODO: simplify or extract
                                        //

                                        view! {
                                            <div class="flex flex-1 center">
                                                <div class="grid gap-0" style:grid-template-columns="1fr 2em">
                                                    {attendance
                                                        .meals
                                                        .iter()
                                                        .map(|meal| {
                                                            let status = local_attendance
                                                                .attendance
                                                                .get(&day)
                                                                .and_then(|d| d.get(&meal.id));
                                                            view! {
                                                                <div
                                                                    class="interactive padded rounded no-select"
                                                                    class:outline-green=status
                                                                        .map(|e| *e == EffectiveAttendance::Present)
                                                                        .unwrap_or(true)
                                                                    class:outline-red=status
                                                                        .map(|e| *e == EffectiveAttendance::Absent)
                                                                        .unwrap_or(false)
                                                                    class:outline-yellow=status
                                                                        .map(|e| *e == EffectiveAttendance::Cancelled)
                                                                        .unwrap_or(false)
                                                                    class:outline-gray=status
                                                                        .map(|e| *e == EffectiveAttendance::Blocked)
                                                                        .unwrap_or(false)
                                                                    on:mousedown=|e| {
                                                                        e.stop_propagation();
                                                                    }
                                                                >
                                                                    {meal.name.clone()}
                                                                </div>
                                                                <div
                                                                    class="padded no-select interactive rounded"
                                                                    on:mousedown=|e| {
                                                                        e.stop_propagation();
                                                                    }
                                                                >
                                                                    {format!(
                                                                        "{}",
                                                                        attendance
                                                                            .attendance
                                                                            .get(&day)
                                                                            .and_then(|m| m.get(&meal.id))
                                                                            .map(|i| *i)
                                                                            .unwrap_or(0),
                                                                    )}
                                                                </div>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </div>
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
            </div>
        </div>
        <Modal is_open=move || meal_edit().is_some() on_close=move || set_meal_edit(None)>
            {
                let meals = attendance.meals.clone();
                move || match meal_edit() {
                    Some(days) => {
                        Either::Left(
                            view! {
                                <MealEditModal
                                    days
                                    meals=meals.clone()
                                    target
                                    on_close=move |_| { set_meal_edit(None) }
                                />
                            },
                        )
                    }
                    None => Either::Right(view! {}),
                }
            }
        </Modal>
    }
}
