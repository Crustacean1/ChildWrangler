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
        <div class="grid-2 gap">
            {attendance
                .attendance
                .into_iter()
                .map(|(name, value))| {
                    view! {
                        <a class="rounded " href=format!("/attendance/{}", id)>
                            {format!("{}", name)}
                        </a>
                        <span>{value}</span>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
