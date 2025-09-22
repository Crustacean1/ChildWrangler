use chrono::NaiveDate;
use dto::attendance::{AttendanceHistoryItemDto, GetAttendanceHistoryDto};
use leptos::prelude::*;
use uuid::Uuid;

use crate::services::attendance::get_attendance_history;

#[component]
pub fn MealHistoryModal(meal_id: Uuid, target: Uuid, date: NaiveDate) -> impl IntoView {
    let history = Resource::new(
        || (),
        move |_| async move {
            get_attendance_history(GetAttendanceHistoryDto {
                meal_id,
                target,
                date,
            })
            .await
        },
    );

    view! {
        <Suspense fallback=|| view!{<div>Loading</div>}>
            <ErrorBoundary fallback=|_| view!{<div>Error</div>}>
                {Suspend::new(async move {
            let history = history.await?;
            Ok::<_,ServerFnError>(view!{})
        })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn MealHistoryModalInner(history: Vec<AttendanceHistoryItemDto>) -> impl IntoView {}
