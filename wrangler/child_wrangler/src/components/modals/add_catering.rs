use std::iter;

use chrono::{NaiveDate, NaiveTime, Weekday};
use dto::catering::{CreateCateringDto, MealDto};
use leptos::{either::Either, logging::log, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        dropdown::Dropdown,
        modal::Modal,
        snackbar::{use_snackbar, SnackbarContext},
    },
    icons::close::CloseIcon,
    services::{catering::create_catering, student::get_meals},
};

#[component]
pub fn AddCateringModal(
    is_open: impl Fn() -> bool + Send + Sync + Copy + 'static,
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let meals = Resource::new(|| (), |_| async move { get_meals().await });

    view! {
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! { <div></div> }
            }>
                {move || Suspend::new(async move {
                    let meals = meals.await;
                    meals.map(|meals| view! { <InnerCateringModal is_open on_close meals /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn InnerCateringModal(
    is_open: impl Fn() -> bool + Send + Sync + Copy + 'static,
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
    meals: Vec<MealDto>,
) -> impl IntoView {
    let snackbar = use_snackbar();
    let on_cancel = move |_| on_close(None);

    let (selected_meals, set_selected_meals) = signal(vec![]);
    let (name, set_name) = signal(String::new());
    let (start, set_start) = signal(String::new());
    let (end, set_end) = signal(String::new());
    let (grace, set_grace) = signal(String::new());

    let on_meal_select = move |meal| {
        match meal {
            Ok(meal) => set_selected_meals.write().push(meal),
            Err(name) => {
                set_selected_meals.write().push(MealDto {
                    id: Uuid::new_v4(),
                    name,
                });
            }
        };
        Some(String::new())
    };

    let (dow, set_dow) = signal(
        iter::successors(Some(Weekday::Mon), |w| {
            if *w == Weekday::Sun {
                None
            } else {
                Some(w.succ())
            }
        })
        .map(|w| (w, false))
        .collect::<Vec<_>>(),
    );

    let on_remove = move |id| set_selected_meals.write().retain(|x| x.id != id);

    let create_catering = Action::new(move |dto: &CreateCateringDto| {
        let dto = dto.clone();
        async move {
            let id = create_catering(dto).await;
            match id {
                Ok(id) => {
                    snackbar.success("Dodano nowy catering");
                    on_close(Some(id));
                }
                Err(e) => snackbar.error("Nie udało się stworzyć cateringu", e),
            }
        }
    });

    let on_save = move |_| match (
        NaiveDate::parse_from_str(&start(), "%Y-%m-%d"),
        NaiveDate::parse_from_str(&end(), "%Y-%m-%d"),
        NaiveTime::parse_from_str(&grace(), "%H:%M"),
    ) {
        (Ok(since), Ok(until), Ok(grace_period)) => {
            let dto = CreateCateringDto {
                name: name(),
                since,
                until,
                grace_period,
                meals: selected_meals().iter().map(|m| m.name.clone()).collect(),
                dow: dow()
                    .into_iter()
                    .map(|(_, enabled)| enabled)
                    .collect::<Vec<_>>(),
            };
            create_catering.dispatch(dto);
        }
        _ => {
            snackbar.error("Podano nieprawidłowy czas lub datę", "");
        }
    };

    view! {
        <div class="gap vertical" style:width="25em">
            <h2 class="h2">Dodaj catering</h2>
            <div class="vertical">
                <label for="name">Nazwa</label>
                <input bind:value=(name, set_name) class="padded rounded" id="name" />
            </div>

            <div class="horizontal gap ">
                <div class="vertical flex-1">
                    <label for="start">Początek</label>
                    <input
                        bind:value=(start, set_start)
                        id="start"
                        class="padded rounded"
                        type="date"
                    />
                </div>
                <div class="vertical flex-1">
                    <label for="end">Koniec</label>
                    <input bind:value=(end, set_end) id="end" class="padded rounded" type="date" />
                </div>
            </div>

            <div style:gap="0.5em" class="vertical">
                <label for="meals">Posiłki</label>
                {move || {
                    if selected_meals().is_empty() {
                        Either::Left(
                            view! {
                                <div class="padded rounded dashed gray">
                                    "Nie wybrano posiłków"
                                </div>
                            },
                        )
                    } else {
                        Either::Right(view! {})
                    }
                }}
                <For each=selected_meals key=|meal: &MealDto| meal.id let:meal>
                    <div class="rounded padded background-3 flex space-between">
                        {meal.name}
                        <button
                            class="interactive red rounded flex icon"
                            on:click=move |_| on_remove(meal.id)
                        >
                            <CloseIcon />
                        </button>
                    </div>
                </For>

                <Dropdown
                    name="Posiłki"
                    options=move || { meals.clone() }
                    key=|i| i.id
                    on_select=on_meal_select
                    item_view=|item| view! { <div class="padded rounded">{item.name}</div> }
                    filter=|needle, hay| hay.name.to_lowercase().contains(&needle.to_lowercase())
                />
            </div>

            <div class="vertical">
                <label for="cancellation">Czas na odmowę</label>
                <input
                    bind:value=(grace, set_grace)
                    id="cancellation"
                    class="padded rounded"
                    type="time"
                    placeholder="Koniec"
                />
            </div>

            <label>Dni tygodnia</label>
            <div class="horizontal" style:gap="0.5em">
                {move || {
                    dow()
                        .iter()
                        .enumerate()
                        .map(|(i, (w, enabled))| {
                            view! {
                                <button
                                    class:green-outline=*enabled
                                    class="interactive padded rounded flex-1"
                                    on:click=move |_| set_dow.write()[i].1 = !dow()[i].1
                                >
                                    {format!("{}", w)}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            <div class="horizontal flex-end gap">
                <button
                    class="padded rounded interactive red"
                    on:click=on_cancel
                    disabled=create_catering.pending()
                >
                    Anuluj
                </button>
                <button
                    class="padded rounded interactive green"
                    on:click=on_save
                    disabled=create_catering.pending()
                >
                    Dodaj
                </button>
            </div>
        </div>
    }
}
