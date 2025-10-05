use chrono::NaiveDate;
use dto::attendance::{AttendanceBreakdownDto, GetAttendanceBreakdownDto};
use leptos::prelude::*;
use uuid::Uuid;

use crate::{components::loader::Loader, services::attendance::get_attendance_breakdown};

#[component]
pub fn MealCountModal(target: Uuid, meal_id: Uuid, date: NaiveDate) -> impl IntoView {
    let attendance = Resource::new(
        || (),
        move |_| async move {
            get_attendance_breakdown(GetAttendanceBreakdownDto {
                target,
                meal_id,
                date,
            })
            .await
        },
    );

    view! {
        <Loader>
                {Suspend::new(async move {
                    let attendance = attendance.await?;
                    Ok::<
                        _,
                        ServerFnError,
                    >(view! { <MealCountModalInner target meal_id date attendance /> })
                })}
        </Loader>
    }
}

#[component]
pub fn MealCountModalInner(
    target: Uuid,
    meal_id: Uuid,
    date: NaiveDate,
    attendance: AttendanceBreakdownDto,
) -> impl IntoView {
    view! {
        <table>
        <thead>
        <tr>
        <th></th>
        <th></th>
        <th></th>
        </tr>
        </thead>
        <tbody class="padded-table">
            {attendance
                .attendance
                .into_iter()
                .map(|(name, (id,attendance, total))| {
                    view! {
                        <tr>
                        <td>
                        <a class="rounded " href=format!("/attendance/{}", id)>
                            {format!("{}", name)}
                        </a>
                        </td>
                        <td>{attendance}</td>
                        <td>{total}</td>
                        </tr>
                    }
                })
                .collect::<Vec<_>>()}
        </tbody>
        </table>
    }
}
