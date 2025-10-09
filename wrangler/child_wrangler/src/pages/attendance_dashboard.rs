use std::f32::consts::PI;

use chrono::Utc;
use dto::attendance::{AttendanceOverviewDto, AttendanceOverviewType};
use dto::catering::CateringDto;
use leptos::prelude::*;
use uuid::Uuid;
use web_sys::wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::components::dropdown::Dropdown;
use crate::components::loader::Loader;
use crate::services::attendance::get_attendance_overview;
use crate::services::catering::get_caterings;

#[component]
pub fn Chart(
    mut padding: i32,
    name: String,
    series: Vec<(AttendanceOverviewType, i32)>,
) -> impl IntoView {
    if series.len() == 1 {
        padding = 1;
    }

    let range = (360 - series.len() * padding as usize) as f32 / 360 as f32;
    let scalar = 2.0 * PI * (padding as f32 / 360 as f32);
    let total = series.iter().map(|(_, s)| s).sum::<i32>() as f32;

    let sizes = series
        .iter()
        .map(|(_, s)| (*s as f32) / total)
        .scan(0.0, |state, x| {
            *state += x;
            Some(*state * 2.0 * PI * range)
        })
        .collect::<Vec<_>>();

    let radius = 80.0;

    let (position, set_position) = signal(None::<(usize, i32, i32)>);

    let colour = |att_type: &AttendanceOverviewType| match att_type {
        AttendanceOverviewType::Present => "green",
        AttendanceOverviewType::Cancelled => "yellow",
        AttendanceOverviewType::Disabled => "red",
        AttendanceOverviewType::Allergic(items) => "blue",
    };

    let title = |att_type: &AttendanceOverviewType| match att_type {
        AttendanceOverviewType::Present => "Obecni",
        AttendanceOverviewType::Cancelled => "Odmówieni",
        AttendanceOverviewType::Disabled => "Nadpisani",
        AttendanceOverviewType::Allergic(items) => "Alergicy",
    };

    let attendance_sum = series
        .iter()
        .filter_map(|(kind, cnt)| match kind {
            AttendanceOverviewType::Present => Some(cnt),
            AttendanceOverviewType::Cancelled => None,
            AttendanceOverviewType::Disabled => None,
            AttendanceOverviewType::Allergic(items) => Some(cnt),
        })
        .sum::<i32>();

    view! {
        <div class="horizontal-wrap relative rounded background-2 gap align-center padded flex-1 align-center justify-center flex-1 overflow-hidden">
            <svg viewBox="-100 -100 200 200" style:height="100%" style:aspect-ratio="1">
                <defs>
                    <filter id="gaussian-1">
                        <feGaussianBlur in="SourceGraphic" stdDeviation="4" result="gauss" />
                        <feMerge>
                            <feMergeNode in="gauss" />
                            <feMergeNode in="SourceGraphic" />
                        </feMerge>
                    </filter>
                </defs>
                <g filter="url(#gaussian-1)" class="anchor-start">
                    {[0.0]
                        .iter()
                        .chain(sizes.iter())
                        .zip(sizes.iter())
                        .enumerate()
                        .map(|(i, (start, end))| {
                            let start = start + i as f32 * scalar;
                            let end = end + i as f32 * scalar;
                            let path = format!(
                                "M {} {} A {} {} 0 {} 1 {} {}",
                                (start).cos() * radius,
                                (start).sin() * radius,
                                radius,
                                radius,
                                (end - start > PI) as i32,
                                (end).cos() * radius,
                                (end).sin() * radius,
                            );
                            view! {
                                <path
                                    d=path
                                    stroke=colour(&series[i].0)
                                    stroke-width="12"
                                    fill="none"
                                    stroke-linecap="round"
                                    on:mousemove=move |e| {
                                        if let Ok(e) = e.dyn_into::<MouseEvent>() {
                                            set_position(Some((i, e.layer_x(), e.layer_y())));
                                        }
                                    }
                                    on:mouseout=move |_| set_position(None)
                                />
                            }
                        })
                        .collect::<Vec<_>>()}
                </g>
                <text
                    x="0"
                    y="1em"
                    text-anchor="middle"
                    fill="gray"
                    font-size="1em"
                    dominant-baseline="middle"
                >
                    {format!("{}", name)}
                </text>
                <text
                    x="0"
                    fill="white"
                    y="-0.25em"
                    text-anchor="middle"
                    font-size="3em"
                    dominant-baseline="middle"
                >
                    {format!("{}", attendance_sum)}
                </text>
            </svg>
            <div class="grid-2 gap align-start justify-center">
                {series
                    .iter()
                    .enumerate()
                    .map(|(i, (name, value))| {
                        view! {
                            <div class="rounded-decoration" style:--background-color={colour(name)}>{title(name)} </div><div>{format!("{}", value)}</div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn AttendanceDashboard() -> impl IntoView {
    let (selected_catering, set_selected_catering) = signal(None::<Uuid>);
    let caterings = Resource::new(|| (), |_| async move { get_caterings().await });

    Effect::new(move |_| {
        caterings
            .get()
            .map(|c| c.map(|c| set_selected_catering(Some(c[0].id))))
    });

    let overview = Resource::new(selected_catering, |catering| async move {
        if let Some(catering) = catering {
            get_attendance_overview(Utc::now().date_naive(), catering).await
        } else {
            Ok(AttendanceOverviewDto {
                meal_list: vec![],
                student_list: Default::default(),
                attendance: Default::default(),
            })
        }
    });

    let on_select = move |item: Result<CateringDto, _>| match item {
        Ok(item) => {
            set_selected_catering(Some(item.id));
            Some(item.name)
        }
        Err(s) => Some(s),
    };

    view! {
        <Loader>

            {move || Suspend::new(async move {
                let caterings = caterings.await?;
                let attendance = overview.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(
                    view! {
                        <div class="horizontal padded rounded background-2">
                            <Dropdown
                                name="Cateringi"
                                options=move || caterings.clone()
                                key=|c| c.id
                                filter=|a, b| true
                                on_select
                                item_view=|catering| {
                                    view! {
                                        <div class="horizontal padded align-center">
                                            {catering.name}
                                        </div>
                                    }
                                }
                            />
                            <h2 class="h2 flex-1">{format!("{}", Utc::now().date_naive())}</h2>
                            <div class="flex-1"></div>
                        </div>
                        <div class="flex-1 column-grid-3">
                            <AttendanceDashboardInner attendance />
                        </div>
                    },
                )
            })}

        </Loader>
    }
}

#[component]
pub fn AttendanceDashboardInner(attendance: AttendanceOverviewDto) -> impl IntoView {
    let meals = attendance.meal_list.into_iter().map(|(id, name)| {
        (
            name,
            attendance.student_list.get(&id).clone(),
            attendance.attendance.get(&id).clone(),
        )
    });

    let attendance_map = |present: bool| {
        view! {
            <span class:red=!present class:green=present>
                {if present { String::from("tak") } else { String::from("nie") }}
            </span>
        }
    };

    meals
        .map(|(meal_name, student_list, att)| {
            view! {
                <div class="vertical rounded gap overflow-hidden gap">
                    <Chart
                        name=meal_name
                        padding=12
                        series=att
                            .map(|att| {
                                att.iter()
                                    .map(|(status, count)| (status.clone(), *count as i32))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or(vec![])
                    />

                    <div class="table-wrapper flex-2 horizontal overflow-hidden rounded background-2">
                        <table class="background-3 rounded flex-1 rounded">
                            <thead>
                                <tr>
                                    <th class="padded">Imię</th>
                                    <th class="padded">Nazwisko</th>
                                    <th class="padded">Grupa</th>
                                    <th class="padded">Obecny</th>
                                </tr>
                            </thead>
                            <tbody class="background-1">
                                {student_list
                                    .map(|list| {
                                        list.iter()
                                            .map(|(id, name, surname, present)| {
                                                view! {
                                                    <tr>
                                                        <td class="padded">{format!("{}", name)}</td>
                                                        <td class="padded">{format!("{}", surname)}</td>
                                                        <td class="padded">N/A</td>
                                                        <td class="padded">{attendance_map(*present)}</td>
                                                    </tr>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or(vec![])}
                            </tbody>
                        </table>
                    </div>
                </div>
            }
        })
        .collect::<Vec<_>>()
}
