use leptos::prelude::*;
use leptos_router::{hooks::use_params, params::Params};

use crate::{
    components::{
        messages::Messages, modal::Modal,
        modals::add_guardian_from_phone::AddGuardianFromPhoneModal,
    },
    icons::{add::AddIcon, person::PersonIcon},
};

#[derive(Params, PartialEq)]
pub struct UnknownParams {
    pub phone: String,
}

#[component]
pub fn UnknownContactDetails() -> impl IntoView {
    let (new_guardian, set_new_guardian) = signal(false);

    let params = use_params::<UnknownParams>();
    let params = move || params.read();

    let phone = move || params().as_ref().ok().map(|params| params.phone.clone());

    view! {
        <div class="flex flex-col flex-1">
            <div class="card flex flex-row align-center p-1">
                <h2 class="flex-1">Nieznany numer</h2>
                <button class="btn">
                    <PersonIcon />
                </button>
                <button class="btn" on:click=move |_| set_new_guardian(true)>
                    <AddIcon />
                </button>
            </div>
            {move || phone().map(|phone| view! { <Messages phone /> })}
        </div>
        <Modal is_open=new_guardian on_close=move || set_new_guardian(false)>
            {move || {
                phone()
                    .map(|phone| {
                        view! {
                            <AddGuardianFromPhoneModal
                                phone
                                on_close=move |_| set_new_guardian(false)
                            />
                        }
                    })
            }}
        </Modal>
    }
}
