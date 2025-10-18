use chrono::NaiveDate;
use dto::attendance::{CateringMealDto, UpdateAttendanceDto};
use leptos::{logging::log, prelude::*};
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::attendance::update_attendance,
};

#[component]
pub fn MealEditModal(
    target: Uuid,
    days: Vec<NaiveDate>,
    meals: Vec<CateringMealDto>,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let (meals, set_meals) = signal(meals.into_iter().map(|m| (m, 0)).collect::<Vec<_>>());
    let (note, set_note) = signal(String::new());

    let changed = move || meals().iter().any(|m| m.1 != 0);

    //TODO: this can be done better (without redrawing all meals) but would require array of
    //signals

    let edit_meal = Action::new({
        let days = days.clone();
        move |_: &()| {
            let days = days.clone();
            let dto = UpdateAttendanceDto {
                target,
                days,
                inactive_meals: meals()
                    .into_iter()
                    .filter(|m| m.1 == 1)
                    .map(|m| m.0.id)
                    .collect::<Vec<_>>(),
                active_meals: meals()
                    .into_iter()
                    .filter(|m| m.1 == 2)
                    .map(|m| m.0.id)
                    .collect::<Vec<_>>(),
                note: note(),
            };
            async move {
                match update_attendance(dto).await {
                    Ok(()) => {
                        snackbar.success("Zaktualizowano obecność");
                        log!("Done, and said");
                        on_close(true)
                    }
                    Err(e) => {
                        snackbar.error("Nie udało się zaktualizowac obecności", e);
                    }
                }
            }
        }
    });

    let on_save = move |_| {
        edit_meal.dispatch(());
    };

    view! {
        <div class="flex flex-col gap-2">
            <h2 class="text-center text-xl">{format!("Edytujesz {} dni", days.len())}</h2>

            {move || {
                meals()
                    .into_iter()
                    .map(move |meal| {
                        view! {
                            <button
                                class="rounded-md p-1 bg-gray-700 md:cursor-pointer md:hover:bg-gray-600"
                                class:outline-2=meal.1 != 0
                                class:outline-green-900=meal.1 == 2
                                class:outline-red-900=meal.1 == 1
                                on:click=move |_| {
                                    let mut meals = meals().clone();
                                    meals
                                        .iter_mut()
                                        .find(|m| m.0.id == meal.0.id)
                                        .map(|m| {
                                            m.1 = (m.1 + 1) % 3;
                                        });
                                    set_meals(meals);
                                }
                            >
                                {meal.0.name.clone()}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
            }}

            <div class="flex flex-col">
                <label>Notka</label>
                <input class="p-1 rounded-md bg-gray-600" bind:value=(note, set_note) type="textfield" />
            </div>
            <div class="flex flex-row gap-2 justify-end">
                <button
                    class="p-2 text-red-900 outline outline-red-900 rounded-md md:hover:bg-gray-700 cursor-pointer"
                    on:click=move |_| on_close(false)
                    disabled=edit_meal.pending()
                >
                    Anuluj
                </button>
                <button
                    class="p-2 text-green-900 outline outline-green-900 rounded-md md:hover:bg-gray-700 cursor-pointer"
                    on:click=on_save
                    disabled=move || edit_meal.pending()() || !changed()
                >
                    Zapisz
                </button>
            </div>
        </div>
    }
}
