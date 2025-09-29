use chrono::NaiveDate;
use dto::attendance::{AttendanceBreakdownDto, GetAttendanceBreakdownDto};
use leptos::prelude::*;
use uuid::Uuid;

use crate::services::attendance::get_attendance_breakdown;

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
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {Suspend::new(async move {
                    let attendance = attendance.await?;
                    Ok::<
                        _,
                        ServerFnError,
                    >(view! { <MealCountModalInner target meal_id date attendance /> })
                })}
            </ErrorBoundary>
        </Suspense>
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
        <h2 class="h2">{format!("Obecność  {}", date)}</h2>
        <h3 class="h2">{format!("Posiłek: {}", attendance.meal)}</h3>
        <div class="grid-2 gap">
            {attendance
                .attendance
                .into_iter()
                .map(|(id, (name, value))| {
                    view! {
                        <a class="interactive rounded padded" href=format!("/attendance/{}", id)>
                            {format!("{}", name)}
                        </a>
                        <span class="padded">{value}</span>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
