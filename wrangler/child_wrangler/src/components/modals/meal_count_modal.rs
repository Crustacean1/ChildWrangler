use chrono::Datelike;
use chrono::NaiveDate;
use dto::attendance::{AttendanceBreakdownDto, GetAttendanceBreakdownDto};
use leptos::prelude::*;
use uuid::Uuid;

use crate::{components::loader::Loader, services::attendance::get_attendance_breakdown};

#[component]
pub fn MealCountModal(target: Uuid, date: NaiveDate) -> impl IntoView {
    let attendance = Resource::new(
        || (),
        move |_| async move {
            get_attendance_breakdown(GetAttendanceBreakdownDto { target, date }).await
        },
    );

    view! {
        <div class="flex flex-col gap-2">
            <Loader>
                {Suspend::new(async move {
                    let attendance = attendance.await?;
                    Ok::<_, ServerFnError>(view! { <MealCountModalInner target date attendance /> })
                })}
            </Loader>
        </div>
    }
}

#[component]
pub fn MealCountModalInner(
    target: Uuid,
    date: NaiveDate,
    attendance: AttendanceBreakdownDto,
) -> impl IntoView {
    view! {
        <table class="table-auto text-center border border-gray-300 border-collapse w-full">
            <thead class="bg-gray-700 font-normal">
                <tr>
                    <th class="font-normal p-4 border border-gray-300/25">Grupa</th>
                    {attendance
                        .meals
                        .iter()
                        .map(|meal| {
                            view! {
                                <th class="font-normal p-4 border border-gray-300/25">
                                    {meal.name.clone()}
                                </th>
                            }
                        })
                        .collect::<Vec<_>>()}
                    <th class="font-normal p-4 border border-gray-300/25">Total</th>
                </tr>
            </thead>
            <tbody class="bg-gray-800 text-gray-400">
                {attendance
                    .groups
                    .into_iter()
                    .map(|group| {
                        view! {
                            <tr class="text-sm">
                                <td class="p-4 border border-gray-300/25">
                                    <a href=format!(
                                        "/attendance/{}/{}/{}",
                                        group.id,
                                        date.year(),
                                        date.month(),
                                    )>{format!("{}", group.name)}</a>
                                </td>
                                {attendance
                                    .meals
                                    .iter()
                                    .map(|meal| {
                                        view! {
                                            <td class="p-4 border border-gray-300/25">
                                                {if let Some((total, present)) = attendance
                                                    .attendance
                                                    .get(&group.id)
                                                    .and_then(|group_meals| group_meals.get(&meal.id))
                                                {
                                                    format!("{}", present)
                                                } else {
                                                    format!("0")
                                                }}
                                            </td>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                                <td class="p-4 border border-gray-300/25">
                                    {attendance
                                        .attendance
                                        .get(&group.id)
                                        .and_then(|meals| {
                                            meals.iter().next().map(|meal| format!("{}", meal.1.0))
                                        })
                                        .unwrap_or(format!("0"))}
                                </td>
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}
