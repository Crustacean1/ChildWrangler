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
        <div class="vertical gap-0">
            <h2 class="h2">{format!("Edytujesz {} dni", days.len())}</h2>

            {move || {
                meals()
                    .into_iter()
                    .map(move |meal| {
                        view! {
                            <button
                                class="interactive padded rounded"
                                class:outline-green=meal.1 == 2
                                class:outline-red=meal.1 == 1
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

            <div class="vertical">
                <label>Notka</label>
                <input class="padded rounded" bind:value=(note, set_note) type="textfield" />
            </div>
            <div class="horizontal gap flex-end">
                <button
                    class="interactive padded rounded red"
                    on:click=move |_| on_close(false)
                    disabled=edit_meal.pending()
                >
                    Anuluj
                </button>
                <button
                    class="interactive padded rounded green"
                    on:click=on_save
                    disabled=move || edit_meal.pending()() || !changed()
                >
                    Zapisz
                </button>
            </div>
        </div>
    }
}
