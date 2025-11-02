use chrono::{Datelike, Utc};
use dto::messages::GuardianDetails;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use uuid::Uuid;

use crate::components::messages::Messages;
use crate::components::modal::Modal;
use crate::components::modals::modify_guardian_phone::ModifyGuardianModal;
use crate::icons::edit::EditIcon;
use crate::icons::person::PersonIcon;
use crate::icons::phone::PhoneIcon;
use crate::services::messages::get_guardian_details;

#[derive(Params, PartialEq)]
pub struct GuardianParams {
    pub id: Uuid,
}

#[component]
pub fn GuardianContactDetails() -> impl IntoView {
    let params = use_params::<GuardianParams>();
    let params = move || params.read();

    let id = move || {
        params()
            .as_ref()
            .ok()
            .map(|params| params.id)
            .unwrap_or(Uuid::nil())
    };
    let details = Resource::new(
        move || id(),
        |id| async move { get_guardian_details(id).await },
    );

    view! {
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let details = details.await?;
                    Ok::<_, ServerFnError>(view! { <InnerGuardianContactDetails details /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn InnerGuardianContactDetails(details: GuardianDetails) -> impl IntoView {
    let (edit_guardian, set_edit_guardian) = signal(false);
    let now = Utc::now();

    view! {
        <div class="flex flex-col gap-2 flex-1">
            <div class="flex flex-row card p-1 gap-2 items-center">
                <h2 class="text-lg">{format!("{}", details.fullname)}</h2>
                {if let Some(phone) = details.phone.clone() {
                    Either::Left(
                        view! {
                            <span class="flex flex-row align-center">
                                {format!("{}", phone)} <PhoneIcon />
                            </span>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <span class="flex flex-row items-center outline outline-red-800 text-red-800 rounded-md bg-red-800/15 p-1">
                                Nie podano numeru telefonu
                            </span>
                        },
                    )
                }}
                {details
                    .students
                    .iter()
                    .map(|student| {
                        view! {
                            <a
                                href=format!(
                                    "/attendance/{}/{}/{}",
                                    student.id,
                                    now.year(),
                                    now.month(),
                                )
                                class="rounded-full p-1 outline outline-green-800 bg-green-800/25 flex flex-row pr-2 pl-2"
                            >
                                <PersonIcon />
                                {format!("{} {}", student.name, student.surname)}
                            </a>
                        }
                    })
                    .collect::<Vec<_>>()}
                <button class="btn justify-self-end" on:click=move |_| set_edit_guardian(true)>
                    <EditIcon />
                </button>
            </div>
            {details.phone.clone().map(|phone| view! { <Messages phone /> })}
        </div>
        <Modal is_open=edit_guardian on_close=move || set_edit_guardian(false)>
            <ModifyGuardianModal
                details=details.clone()
                on_close=move |_| set_edit_guardian(false)
            />
        </Modal>
    }
}
