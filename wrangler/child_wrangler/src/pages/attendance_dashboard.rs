use std::collections::HashMap;
use std::f32::consts::PI;

use chrono::Utc;
use dto::attendance::{AttendanceOverviewDto, AttendanceOverviewType, AttendanceStatus};
use dto::catering::{CateringDto, MealDto};
use dto::group::GroupDto;
use dto::student::{AllergyCombinationDto, StudentDto};
use leptos::either::Either;
use leptos::prelude::*;
use uuid::Uuid;

use crate::components::dropdown::Dropdown;
use crate::components::general_provider::{
    AllergyResource, GroupResource, MealResource, StudentResource,
};
use crate::components::loader::Loader;
use crate::services::attendance::get_attendance_overview;
use crate::services::catering::get_caterings;

#[component]
pub fn Chart(
    mut padding: i32,
    name: String,
    allergies: HashMap<Uuid, AllergyCombinationDto>,
    series: Vec<(AttendanceOverviewType, i64)>,
) -> impl IntoView {
    if series.len() == 1 {
        padding = 1;
    }

    let range = (360 - series.len() * padding as usize) as f32 / 360 as f32;
    let scalar = 2.0 * PI * (padding as f32 / 360 as f32);
    let total = series.iter().map(|(_, s)| s).sum::<i64>() as f32;

    let sizes = series
        .iter()
        .map(|(_, s)| (*s as f32) / total)
        .scan(0.0, |state, x| {
            *state += x;
            Some(*state * 2.0 * PI * range)
        })
        .collect::<Vec<_>>();

    let radius = 80.0;

    let colour = |att_type: &AttendanceOverviewType| match att_type {
        AttendanceOverviewType::Present(id) => {
            if allergies
                .get(&id)
                .map(|allergies| allergies.allergies.is_empty())
                .unwrap_or(true)
            {
                "green"
            } else {
                "blue"
            }
        }
        AttendanceOverviewType::Cancelled => "yellow",
        AttendanceOverviewType::Disabled => "red",
    };

    let title = |att_type: &AttendanceOverviewType| match att_type {
        AttendanceOverviewType::Present(id) => {
            let allergies = allergies
                .get(&id)
                .map(|allergies| allergies.allergies.clone())
                .unwrap_or(vec![]);
            if allergies.is_empty() {
                "Obecni".into()
            } else {
                format!("Alergicy [{}]", allergies.join(", "))
            }
        }
        AttendanceOverviewType::Cancelled => "Odmówieni".into(),
        AttendanceOverviewType::Disabled => "Nadpisani".into(),
    };

    let attendance_sum = series
        .iter()
        .filter_map(|(kind, cnt)| match kind {
            AttendanceOverviewType::Present(_) => Some(cnt),
            AttendanceOverviewType::Cancelled => None,
            AttendanceOverviewType::Disabled => None,
        })
        .sum::<i64>();

    view! {
        <div class="flex-1 flex flex-row flex-wrap min-w-72 justify-center items-center">
            <svg viewBox="-100 -100 200 200" class="aspect-ratio-1 min-w-32 max-w-96 flex-1">
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
                            let start = start + i as f32 * scalar + scalar * 0.5;
                            let end = end + i as f32 * scalar + scalar * 0.5;
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
                                    stroke-width="2"
                                    fill="none"
                                    stroke-linecap="round"
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
            <div class="grid grid-cols-2 gap-2 align-start justify-center card p-2 h-fit">
                {series
                    .iter()
                    .enumerate()
                    .map(|(_, (name, value))| {
                        view! {
                            <div class="legend text-left" style:--background-color={colour(name)}>{title(name)} </div>
                            <div class="text-right">{format!("{}", value)}</div>
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
    let meals = expect_context::<MealResource>().0;
    let students = expect_context::<StudentResource>().0;
    let groups = expect_context::<GroupResource>().0;
    let allergies = expect_context::<AllergyResource>().0;

    Effect::new(move |_| {
        caterings
            .get()
            .map(|c| c.map(|c| c.first().map(|c| set_selected_catering(Some(c.id)))))
    });

    let overview = Resource::new(selected_catering, |catering| async move {
        if let Some(catering) = catering {
            get_attendance_overview(Utc::now().date_naive(), catering).await
        } else {
            Ok(AttendanceOverviewDto {
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
                let meals = meals.await?;
                let students = students.await?;
                let groups = groups.await?;
                let allergies = allergies.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(
                    view! {
                        <div class="flex flex-row align-center card p-1">
                            <Dropdown
                                name="Cateringi"
                                options=move || caterings.clone()
                                key=|c| c.id
                                filter=|a, b| true
                                on_select
                                item_view=|catering| {
                                    view! { <div class="p-1 text-center">{catering.name}</div> }
                                }
                            />
                            <h2 class="text-lg text-center flex-1">
                                {format!("{}", Utc::now().format("%d %B %Y"))}
                            </h2>
                            <div class="flex-1"></div>
                        </div>
                        <AttendanceDashboardInner attendance meals groups students allergies />
                    },
                )
            })}

        </Loader>
    }
}

#[component]
pub fn AttendanceDashboardInner(
    attendance: AttendanceOverviewDto,
    meals: HashMap<Uuid, MealDto>,
    allergies: HashMap<Uuid, AllergyCombinationDto>,
    students: HashMap<Uuid, StudentDto>,
    groups: HashMap<Uuid, GroupDto>,
) -> impl IntoView {
    let available_meals = attendance
        .attendance
        .iter()
        .map(|(meal_id, _)| *meal_id)
        .collect::<Vec<_>>();

    view! {
        <div class="flex flex-row">
            {attendance
                .attendance
                .into_iter()
                .map(|(meal_id, attendance)| {
                    view! {
                        <div class="flex-1 flex-wrap flex flex-row">
                            <Chart
                                name=meals
                                    .get(&meal_id)
                                    .map(|m| m.name.clone())
                                    .unwrap_or(format!("Unknown meal"))
                                allergies=allergies.clone()
                                padding=12
                                series=attendance
                            />
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
        <div class="flex-1 overflow-hidden rounded-md">
            <table class="w-full border-collapse text-left">
                <thead>
                    <tr class="bg-gray-600">
                        <th class="p-2">Imię</th>
                        <th class="p-2">Nazwisko</th>
                        <th class="p-2">Grupa</th>
                        {available_meals
                            .iter()
                            .map(|meal_id| {
                                view! {
                                    <th class="p-2">
                                        {meals
                                            .get(meal_id)
                                            .map(|m| m.name.clone())
                                            .unwrap_or(format!("Unknown meal"))}
                                    </th>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody class="background-1">
                    {attendance
                        .student_list
                        .into_iter()
                        .map(|(student_id, attendance)| {
                            let student = students.get(&student_id).unwrap();
                            let group = groups.get(&student.group_id).unwrap();
                            view! {
                                <tr class="even:bg-gray-800 odd:bg-gray-900">
                                    <td class="p-2 r-border">
                                        <a href=format!(
                                            "attendance/{}",
                                            student.id,
                                        )>{format!("{}", student.name)}</a>
                                    </td>
                                    <td class="p-2 ">
                                        <a href=format!(
                                            "attendance/{}",
                                            student.id,
                                        )>{format!("{}", student.surname)}</a>
                                    </td>
                                    <td class="p-2 ">
                                        <a href=format!(
                                            "attendance/{}",
                                            group.id,
                                        )>{format!("{}", group.name)}</a>
                                    </td>

                                    {available_meals
                                        .iter()
                                        .map(|meal_id| {
                                            view! {
                                                <td class="p-2">
                                                    {match attendance.get(meal_id) {
                                                        Some(AttendanceStatus::Present) => {
                                                            Either::Left(
                                                                Either::Left(
                                                                    view! { <span class="text-green-800">tak</span> },
                                                                ),
                                                            )
                                                        }
                                                        Some(AttendanceStatus::Overriden) => {
                                                            Either::Left(
                                                                Either::Right(
                                                                    view! { <span class="text-red-800">nie</span> },
                                                                ),
                                                            )
                                                        }
                                                        Some(AttendanceStatus::Cancelled) => {
                                                            Either::Right(
                                                                Either::Left(
                                                                    view! { <span class="text-yellow-800">nie</span> },
                                                                ),
                                                            )
                                                        }
                                                        None => Either::Right(Either::Right(view! {})),
                                                    }}
                                                </td>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </tr>
                            }
                        })
                        .collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
