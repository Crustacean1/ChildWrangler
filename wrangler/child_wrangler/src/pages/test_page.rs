use leptos::prelude::*;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::test::generate_random_data,
};

#[component]
pub fn TestPage() -> impl IntoView {
    let (catering_count, set_catering_count) = signal(String::from("5"));
    let (group_count, set_group_count) = signal(String::from("100"));
    let (student_count, set_student_count) = signal(String::from("1000"));
    let (guardian_count, set_guardian_count) = signal(String::from("800"));

    let snackbar = use_snackbar();
    let on_click_action = Action::new(move |_: &()| async move {
        match generate_random_data(
            catering_count().parse::<i32>().unwrap(),
            group_count().parse::<i32>().unwrap(),
            student_count().parse::<i32>().unwrap(),
            guardian_count().parse::<i32>().unwrap(),
        )
        .await
        {
            Ok(_) => snackbar.success("Dodano dane testowe"),
            Err(e) => snackbar.error("Nie udało się dodać danych", e),
        }
    });

    view! {
        <div class="flex flex-col gap">
            <label>Catering count</label>
            <input
                bind:value=(catering_count, set_catering_count)
                class="bg-gray-600 rounded-md p-1"
                type="number"
                name="catering count"
            />
            <label>Group count</label>
            <input
                bind:value=(group_count, set_group_count)
                class="bg-gray-600 rounded-md p-1"
                type="number"
                name="group count"
            />
            <label>Student count</label>
            <input
                bind:value=(student_count, set_student_count)
                class="bg-gray-600 rounded-md p-1"
                type="number"
                name="student count"
            />
            <label>Guardian count</label>
            <input
                bind:value=(guardian_count, set_guardian_count)
                class="bg-gray-600 rounded-md p-1"
                type="number"
                name="guardian count"
            />
            <button
                class="md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                on:click=move |_| {
                    on_click_action.dispatch(());
                }
            >
                Wygeneruj dane
            </button>
        </div>
    }
}
