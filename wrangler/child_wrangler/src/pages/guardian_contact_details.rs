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

    view! {
        <div class="vertical gap flex-1">
            <div class="background-2 horizontal padded rounded">
                {if let Some(phone) = details.phone.clone() {
                    Either::Left(
                        view! {
                            <h2 class="h2 flex-1 text-left horizontal gap align-center">
                                <span>{format!("{} ", details.fullname)}</span>
                                <span class="horizontal align-center">
                                    {format!("{}", phone)} <PhoneIcon />
                                </span>
                            </h2>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <h2 class="h2 flex-1 text-left">{format!("{}", details.fullname)}</h2>
                        },
                    )
                }}
                <button
                    class="self-end interactive icon-button"
                    on:click=move |_| set_edit_guardian(true)
                >
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
