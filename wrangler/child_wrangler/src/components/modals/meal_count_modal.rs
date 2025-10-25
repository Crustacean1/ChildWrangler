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
        <h2 class="text-center text-lg">
        {format!("{} Obecność dla grupy {}", date, target)}
        </h2>
        <Loader>
            {Suspend::new(async move {
                let attendance = attendance.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(view! { <MealCountModalInner target date attendance /> })
            })}
        </Loader>
    }
}

#[component]
pub fn MealCountModalInner(
    target: Uuid,
    date: NaiveDate,
    attendance: AttendanceBreakdownDto,
) -> impl IntoView {
    view! {
        <table class="table-fixed">
            <thead>
                <tr>
                    <th class="p-1">Grupa</th>
                    <th class="p-1">Total</th>
                    {attendance.meals.iter().map(|meal| view!{<th class="p-1">{meal.name.clone()}</th>}).collect::<Vec<_>>()}
                </tr>
            </thead>
            <tbody>
                {attendance
                    .groups
                    .into_iter()
                    .map(|group| {
                        view! {
                            <tr>
                                <td class="p-1">
                                    <a class="rounded " href=format!("/attendance/{}/{}/{}", group.id, date.year(), date.month())>
                                        {format!("{}", group.name)}
                                    </a>
                                </td>
                                {attendance.meals.iter().map(|meal| {
                                view!{
                                    <td>format!("{}/{}", attendance.attendance.get(group.id).and_then(|group_meals| group_meals.get(meal.id)))</td>
                                }
                            }).collect::<Vec<_>>()}
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}
