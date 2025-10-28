use std::iter;

use chrono::{NaiveDate, NaiveTime, Weekday};
use dto::catering::{CreateCateringDto, MealDto};
use leptos::{either::Either, logging::log, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        dropdown::Dropdown,
        general_provider::GroupVersion,
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
    let update_groups = expect_context::<GroupVersion>();
    let on_cancel = move |_| on_close(None);

    let (selected_meals, set_selected_meals) = signal(vec![]);
    let (name, set_name) = signal(String::new());
    let (start, set_start) = signal(String::new());
    let (end, set_end) = signal(String::new());
    let (grace, set_grace) = signal(String::new());

    let on_meal_select = move |meal: Result<MealDto, String>| match meal {
        Ok(meal) => {
            set_selected_meals.write().push(meal);
            None
        }
        Err(name) => {
            let name = name.trim();
            if !selected_meals()
                .iter()
                .any(|m| m.name.to_lowercase() == name.to_lowercase())
                && !name.is_empty()
            {
                set_selected_meals.write().push(MealDto {
                    id: Uuid::new_v4(),
                    name: String::from(name),
                });
                Some(String::from(name))
            } else {
                None
            }
        }
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
                    *update_groups.0.write() += 1;
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

    let available_meals = move || {
        let selected_meals = selected_meals();
        meals
            .clone()
            .into_iter()
            .filter(|m| {
                !selected_meals
                    .iter()
                    .any(|b| b.name.to_lowercase() == m.name.to_lowercase())
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="gap-2 flex flex-col" style:width="25em">
            <h2 class="text-center text-lg">Dodaj catering</h2>
            <div class="flex flex-col">
                <label for="name">Nazwa</label>
                <input
                    data-testid="catering-name"
                    bind:value=(name, set_name)
                    class="p-1 rounded-md bg-gray-600"
                    id="name"
                />
            </div>

            <div class="flex flex-row gap-2">
                <div class="flex flex-col flex-1">
                    <label for="start">Początek</label>
                    <input
                        data-testid="catering-start"
                        bind:value=(start, set_start)
                        id="start"
                        class="p-1 rounded-md bg-gray-600"
                        type="date"
                    />
                </div>
                <div class="flex flex-col flex-1">
                    <label for="end">Koniec</label>
                    <input
                        data-testid="catering-end"
                        bind:value=(end, set_end)
                        id="end"
                        class="p-1 rounded-md bg-gray-600"
                        type="date"
                    />
                </div>
            </div>

            <label for="meals">Posiłki</label>
            <div class="p-1 rounded-md outline outline-dashed outline-gray">
                <div class="flex flex-col gap-2">
                    {move || {
                        if selected_meals().is_empty() {
                            Either::Left(view! { "Nie wybrano posiłków" })
                        } else {
                            Either::Right(view! {})
                        }
                    }} <For each=selected_meals key=|meal: &MealDto| meal.id let:meal>
                        <div class="rounded-md outline outline-stone-300 flex-1 flex align-center p-1">
                            <span class="p-1 flex-1 align-self-center">{meal.name}</span>
                            <button
                                class="p-1 md:hover:bg-gray-700 md:active:bg-gray-600 md:cursor-pointer red rounded-md"
                                on:click=move |_| on_remove(meal.id)
                            >
                                <CloseIcon />
                            </button>
                        </div>
                    </For>
                </div>
            </div>

            <Dropdown
                name="meals"
                options=available_meals
                key=|i| i.id
                on_select=on_meal_select
                item_view=|item| view! { <div class="p-1">{item.name}</div> }
                filter=|needle, hay| hay.name.to_lowercase().contains(&needle.to_lowercase())
            />

            <div class="flex flex-col">
                <label for="cancellation">Czas na odmowę</label>
                <input
                    data-testid="catering-cancellation"
                    bind:value=(grace, set_grace)
                    id="cancellation"
                    class="p-1 rounded-md bg-gray-600"
                    type="time"
                    placeholder="Koniec"
                />
            </div>

            <label>Dni obowiązywania</label>
            <div class="flex flex-row gap-2">
                {move || {
                    dow()
                        .iter()
                        .enumerate()
                        .map(|(i, (w, enabled))| {
                            view! {
                                <button
                                    id=format!("{}", w)
                                    data-testid=format!("dow-{}", w)
                                    class:outline-2=*enabled
                                    class="p-1 rounded-md outline-green-900 md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600 flex-1"
                                    on:click=move |_| set_dow.write()[i].1 = !dow()[i].1
                                >
                                    {format!("{}", w)}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>

            <div class="flex flex-row justify-end gap-2">
                <button
                    data-testid="catering-cancel"
                    class="p-1 rounded-md outline outline-red-800 text-red-800 md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600"
                    on:click=on_cancel
                    disabled=create_catering.pending()
                >
                    Anuluj
                </button>
                <button
                    data-testid="catering-save"
                    class="p-1 rounded-md outline outline-green-800 text-green-800 md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600"
                    on:click=on_save
                    disabled=create_catering.pending()
                >
                    Dodaj
                </button>
            </div>
        </div>
    }
}
