use chrono::NaiveDate;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn MealCountModal(id: Uuid, meal_id: Uuid, date: NaiveDate) -> impl IntoView {
    view! {
        <h2 class="h2">Dużo posiłków</h2>
        <div></div>
    }
}
