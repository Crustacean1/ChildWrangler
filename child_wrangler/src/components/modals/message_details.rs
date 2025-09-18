use leptos::prelude::*;

use crate::{icons::refresh::RefreshIcon, services::messages::get_message_processing_info};

#[component]
pub fn MessageDetailsModal(msg_id: i32) -> impl IntoView {
    let details = Resource::new(
        || (),
        |_| async move { get_message_processing_info(msg_id).await },
    );

    view! {
        <MessageDetailsModalInner/>
    }
}

#[component]
fn MessageDetailsModalInner() -> impl IntoView {
    view! {
        <h2 class="h2 horizontal gap">Szczegóły wiadomości
        <button class="interactive icon-button">
        <RefreshIcon/>
        </button>
        </h2>
    }
}
