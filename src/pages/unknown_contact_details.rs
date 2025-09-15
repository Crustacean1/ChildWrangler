use leptos::prelude::*;
use leptos_router::{hooks::use_params, params::Params};

use crate::{components::messages::Messages, icons::add::AddIcon};

#[derive(Params, PartialEq)]
pub struct UnknownParams {
    pub phone: String,
}

#[component]
pub fn UnknownContactDetails() -> impl IntoView {
    let params = use_params::<UnknownParams>();
    let params = move || params.read();

    let phone = move || params().as_ref().ok().map(|params| params.phone.clone());

    view! {
        <div class="vertical gap flex-1">
            <div class="padded rounded background-2 horizontal align-center text-left">
                <h2 class="h2 flex-1">Nieznany numer</h2>
                <button class="interactive rounded center">
                    <AddIcon />
                </button>
            </div>
            {move || phone().map(|phone| view! { <Messages phone /> })}
        </div>
    }
}
