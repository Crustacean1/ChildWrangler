use leptos::{either::Either, logging::log, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        dropdown::Dropdown,
        snackbar::{use_snackbar, SnackbarContext},
    },
    dtos::{
        catering::{AllergyDto, GuardianDetailDto, GuardianDto},
        details::StudentDetailsDto,
        student::CreateStudentDto,
    },
    icons::close::CloseIcon,
    services::student::{create_student, get_allergies, get_guardians, update_student},
};

#[component]
pub fn AddStudentModal(
    group: Uuid,
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
    initial: Option<StudentDetailsDto>,
) -> impl IntoView {
    let allergies = Resource::new(|| (), |_| async move { get_allergies().await });
    let guardians = Resource::new(|| (), |_| async move { get_guardians().await });

    view! {
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || {
                    let initial = initial.clone();
                    Suspend::new(async move {
                        let allergies = allergies.await?;
                        let guardians = guardians.await?;
                        Ok::<
                            _,
                            ServerFnError,
                        >(
                            view! {
                                <InnerAddStudentModal on_close group allergies guardians initial />
                            },
                        )
                    })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn InnerAddStudentModal(
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
    group: Uuid,
    guardians: Vec<GuardianDto>,
    allergies: Vec<AllergyDto>,
    initial: Option<StudentDetailsDto>,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let (name, set_name) = signal(
        initial
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or(String::new()),
    );
    let (surname, set_surname) = signal(
        initial
            .as_ref()
            .map(|i| i.surname.clone())
            .unwrap_or(String::new()),
    );
    let (selected_allergies, set_selected_allergies) = signal(
        initial
            .as_ref()
            .map(|i| i.allergies.clone())
            .unwrap_or(vec![]),
    );
    let (selected_guardians, set_selected_guardians) = signal(
        initial
            .as_ref()
            .map(|i| i.guardians.clone())
            .unwrap_or(vec![]),
    );

    let available_guardians = move || {
        let g2 = selected_guardians().clone();
        guardians
            .clone()
            .into_iter()
            .filter(|g| !g2.iter().any(|g2: &GuardianDto| g2.id == g.id))
            .collect::<Vec<_>>()
    };

    let available_allergies = move || {
        let a2 = selected_allergies().clone();
        allergies
            .clone()
            .into_iter()
            .filter(|a| !a2.iter().any(|a2: &AllergyDto| a2.id == a.id))
            .collect::<Vec<_>>()
    };

    let on_add_allergy = move |allergy| match allergy {
        Ok(allergy) => set_selected_allergies.write().push(allergy),
        Err(name) => set_selected_allergies.write().push(AllergyDto {
            id: Uuid::new_v4(),
            name,
        }),
    };

    let on_add_guardian = move |guardian| {
        log!("Wtf: {:?}", guardian);
        match guardian {
            Ok(guardian) => set_selected_guardians.write().push(guardian),
            Err(fullname) => set_selected_guardians.write().push(GuardianDto {
                id: Uuid::new_v4(),
                fullname,
            }),
        }
    };

    let update_id = initial.as_ref().map(|i| i.id);

    let save_student = Action::new(move |insert_dto: &CreateStudentDto| {
        let insert_dto = insert_dto.clone();

        let guardians = selected_guardians();
        let allergies = selected_allergies();
        async move {
            if let Some(id) = update_id {
                let dto = StudentDetailsDto {
                    id,
                    name: insert_dto.name,
                    surname: insert_dto.surname,
                    guardians,
                    allergies,
                };
                match update_student(dto).await {
                    Ok(_) => {
                        snackbar.success("Zaktualizowano ucznia");
                        on_close(Some(id))
                    }
                    Err(e) => snackbar.error("Nie udało się zaktualizować ucznia", e),
                }
            } else {
                match create_student(insert_dto).await {
                    Ok(id) => {
                        snackbar.success("Dodano ucznia");
                        on_close(Some(id))
                    }
                    Err(e) => snackbar.error("Nie udało się dodać ucznia", e),
                }
            }
        }
    });

    let on_save = move |_| {
        let dto = CreateStudentDto {
            name: name(),
            group_id: group,
            surname: surname(),
            allergies: selected_allergies().into_iter().map(|a| a.name).collect(),
            guardians: selected_guardians()
                .into_iter()
                .map(|a| a.fullname)
                .collect(),
        };
        save_student.dispatch(dto);
    };

    view! {
        <h2 class="h2">{if update_id.is_none() { "Dodaj ucznia" } else { "Edytuj ucznia" }}</h2>
        <div class="horizontal gap">
            <div class="vertical">
                <label for="name">Imię</label>
                <input bind:value=(name, set_name) id="name" class="padded rounded" />
            </div>
            <div class="vertical">
                <label for="surname">Nazwisko</label>
                <input bind:value=(surname, set_surname) id="surname" class="padded rounded" />
            </div>
        </div>

        <label>Alergie</label>
        <ul
            class="gap vertical padded rounded dashed"
            class:gray=move || selected_allergies().is_empty()
        >
            {move || {
                if selected_allergies().is_empty() {
                    Either::Left(view! { <li class="gray">Brak alergii</li> })
                } else {
                    Either::Right(view! {})
                }
            }}
            <For each=selected_allergies key=|a: &AllergyDto| a.id let:allergy>
                <li class="padded rounded background-3 horizontal flex space-between align-center">
                    {allergy.name} <button class="interactive red rounded flex icon">
                        <CloseIcon on:click=move |_| {
                            set_selected_allergies.write().retain(|a| a.id != allergy.id)
                        } />
                    </button>
                </li>
            </For>
        </ul>
        <Dropdown
            name="alergie"
            options=available_allergies
            key=|a| a.id
            on_select=on_add_allergy
            item_view=|item| view! { <div class="padded rounded">{item.name}</div> }
            filter=|needle, hay| hay.name.to_lowercase().contains(&needle.to_lowercase())
        />

        <label>Rodzice</label>
        <ul
            class="vertical padded gap rounded dashed"
            class:gray=move || selected_guardians().is_empty()
        >
            {move || {
                if selected_guardians().is_empty() {
                    Either::Left(view! { <li>Brak rodziców</li> })
                } else {
                    Either::Right(view! {})
                }
            }}
            <For each=selected_guardians key=|g: &GuardianDto| g.id let:guardian>
                <li class="padded rounded background-3 horizontal flex space-between align-center">
                    {guardian.fullname} <button class="interactive red rounded flex icon">
                        <CloseIcon on:click=move |_| {
                            set_selected_guardians.write().retain(|a| a.id != guardian.id)
                        } />
                    </button>
                </li>
            </For>
        </ul>
        <Dropdown
            name="guardians"
            options=available_guardians
            key=|a| a.id
            on_select=on_add_guardian
            item_view=|item| view! { <div class="padded rounded">{item.fullname}</div> }
            filter=|needle, hay| hay.fullname.to_lowercase().contains(&needle.to_lowercase())
        />

        <div class="horizontal flex-end gap">
            <button
                class="interactive padded rounded red"
                on:click=move |_| on_close(None)
                disabled=save_student.pending()
            >
                Anuluj
            </button>
            <button
                class="interactive padded rounded green"
                on:click=on_save
                disabled=save_student.pending()
            >
                {if update_id.is_some() { "Zapisz" } else { "Dodaj" }}
            </button>
        </div>
    }
}
