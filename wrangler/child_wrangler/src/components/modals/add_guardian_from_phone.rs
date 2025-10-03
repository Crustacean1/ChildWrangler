use dto::student::{CreateGuardianDto, StudentDto};
use leptos::{either::Either, prelude::*};

use crate::{
    components::{
        dropdown::Dropdown,
        snackbar::{use_snackbar, SnackbarContext},
    },
    icons::close::CloseIcon,
    services::student::{create_guardian, get_students},
};

#[component]
pub fn AddGuardianFromPhoneModal(
    phone: String,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let students = Resource::new(|| (), |_| async move { get_students().await });

    view! {
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new({
                    let phone = phone.clone();
                    async move {
                        let students = students.await?;
                        let phone = phone.clone();
                        Ok::<
                            _,
                            ServerFnError,
                        >(

                            view! { <AddGuardianFromPhoneModalInner phone on_close students /> },
                        )
                    }
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn AddGuardianFromPhoneModalInner(
    phone: String,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
    students: Vec<StudentDto>,
) -> impl IntoView {
    let snackbar = use_snackbar();
    let (selected_students, set_selected_students) = signal(vec![]);
    let (fullname, set_fullname) = signal(String::new());

    let save_guardian = Action::new(move |_: &()| {
        let dto = CreateGuardianDto {
            fullname: fullname(),
            phone: phone.clone(),
            students: selected_students()
                .iter()
                .map(|s: &StudentDto| s.id)
                .collect::<Vec<_>>(),
        };
        async move {
            match create_guardian(dto).await {
                Ok(_) => {
                    snackbar.success("Dodano nowego rodzica");
                    on_close(true);
                }
                Err(e) => snackbar.error("Nie udało się dodać rodzica", e),
            }
        }
    });

    let available_students = {
        let students = students.clone();
        move || {
            let selected = selected_students();
            students
                .clone()
                .into_iter()
                .filter(|s| !selected.iter().any(|b: &StudentDto| b.id == s.id))
                .collect::<Vec<_>>()
        }
    };

    let on_select = move |s| {
        match s {
            Ok(s) => set_selected_students.write().push(s),
            Err(_) => {}
        };
        Some(String::new())
    };

    view! {
        <h2 class="h2">Dodaj rodzica</h2>
        <div class="vertical">
            <label>Nazwa</label>
            <input bind:value=(fullname, set_fullname) class="padded rounded" />
        </div>
        <div class="vertical gap">
            <label>Uczniowie</label>
            <ul class="dashed padded rounded gap">
                {move || {
                    if selected_students().is_empty() {
                        Either::Left(view! { <div class="gray">Nie wybrano uczniów</div> })
                    } else {
                        Either::Right(view! {})
                    }
                }} <For each=selected_students key=|s| s.id let:student>
                    <li class="horizontal space-between background-3 align-center rounded">
                        <span class="padded">
                            {format!("{} {}", student.name, student.surname)}
                        </span>
                        <button
                            class="icon-button interactive"
                            on:click=move |_| {
                                set_selected_students.write().retain(|s| s.id != student.id)
                            }
                        >
                            <CloseIcon />
                        </button>
                    </li>
                </For>
            </ul>
            <Dropdown
                name="students"
                options=available_students
                key=|s| s.id
                filter=|n, h| h.name.to_lowercase().contains(&n.to_lowercase())
                on_select
                item_view=|s| {
                    view! { <div class="padded">{format!("{} {}", s.name, s.surname)}</div> }
                }
            />
        </div>
        <div class="horizontal gap flex-end">
            <button
                class="padded rounded interactive red"
                on:click=move |_| on_close(false)
                disabled=save_guardian.pending()
            >
                Anuluj
            </button>
            <button
                class="padded rounded interactive green"
                disabled=save_guardian.pending()
                on:click=move |_| {
                    save_guardian.dispatch(());
                }
            >
                Zapisz
            </button>
        </div>
    }
}
