use std::f32::consts::PI;

use chrono::Utc;
use dto::attendance::AttendanceOverviewDto;
use leptos::logging::log;
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::components::loader::Loader;
use crate::pages::attendance_page::AttendanceParams;
use crate::services::attendance::get_attendance_overview;

#[component]
pub fn Chart(padding: i32, series: Vec<(String, i32)>) -> impl IntoView {
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
    let colors = vec!["#ff0000", "#00ff00", "yellow", "#ffff00"];

    let (position, set_position) = signal(None::<(usize, i32, i32)>);

    view! {
        <div class="relative horizontal gap align-center">
            <svg width="200" height="200" viewBox="-100 -100 200 200">
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
                                    stroke=colors[i]
                                    stroke-width="12"
                                    fill="none"
                                    stroke-linecap="round"
                                    on:mousemove=move |e| {
                                        e.dyn_into::<MouseEvent>()
                                            .map(move |e| set_position(
                                                Some((i, e.layer_x(), e.layer_y())),
                                            ));
                                    }
                                    on:mouseout=move |_| set_position(None)
                                />
                            }
                        })
                        .collect::<Vec<_>>()}
                </g>
                <text
                    x="0"
                    y="0"
                    text-anchor="middle"
                    fill="white"
                    dominant-baseline="middle"
                    font-size="2em"
                >
                {format!("{}", total)}
                </text>
            </svg>
            <div
                class="pretty-background rounded padded anchor-end"
                style:position="absolute"
                style:visibility=move || {
                    position().map_or(String::from("hidden"), |_| String::new())
                }
                style:left=move || {
                    position().map(|(_, x, _)| format!("{}px", x + radius as i32 + 32))
                }
                style:top=move || {
                    position().map(|(_, _, y)| format!("{}px", y + radius as i32 - 16))
                }
            >
                {
                    let series = series.clone();
                    move || position().map(|(i, _, _)| format!("{}", series[i].0))
                }
            </div>
            <div class="grid-2 gap align-start justify-center">
                {series
                    .iter()
                    .enumerate()
                    .map(|(i, (name, value))| {
                        view! {
                            <div class="rounded-decoration" style:--background-color={colors[i]}>{format!("{}", name)} </div><div>{format!("{}", value)}</div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn AttendanceDashboard() -> impl IntoView {
    let overview = Resource::new(
        || Utc::now(),
        |date| async move { get_attendance_overview(date.date_naive()).await },
    );

    view! {
        <Loader>
        {move || Suspend::new(async move {

            let attendance =  overview.await?;
        Ok::<_,ServerFnError>(view!{

            <AttendanceDashboardInner attendance/>
        })
        })}
        </Loader>
    }
}

#[component]
pub fn AttendanceDashboardInner(attendance: AttendanceOverviewDto) -> impl IntoView {
    view! {
        <div>
            <div class="padded vertical rounded background-2">
        {
            attendance.attendance.into_iter().map(|(meal_id,att)| view!{
                <h2 class="h2">{format!("{}",meal_id)}</h2>
                {move || {
                    view! {
                        <Chart
                            padding=12
                            series=
                                {att.iter().map(|(status, count)| (
                        format!("{:?}", status), *count as i32
                    )).collect::<Vec<_>>()}

                        />
                    }
                }}
            }).collect::<Vec<_>>()
        }
            </div>
        </div>
    }
}
