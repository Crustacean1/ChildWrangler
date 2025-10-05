use chrono::{Duration, Utc};
use dto::messages::{GeneralMessageDto, PhoneStatusDto};
use leptos::{either::Either, prelude::*};

use crate::{
    components::loader::Loader,
    services::messages::{get_latest_messages, get_phone_status},
};

#[component]
pub fn MessageDashboard() -> impl IntoView {
    let phone = Resource::new(|| (), move |_| async move { get_phone_status().await });
    let messages = Resource::new(|| (), move |_| async move { get_latest_messages().await });

    view! {
        <div class="vertical gap flex-1">
            <Loader>
        {move || Suspend::new(async move {
            let phone = phone.await?;
            let messages = messages.await?;
            Ok::<_,ServerFnError>(view!{<MessageDashbaordInner phone messages/>})
        })}
        </Loader>
            </div>
    }
}

#[component]
pub fn MessageDashbaordInner(
    phone: Option<PhoneStatusDto>,
    messages: Vec<GeneralMessageDto>,
) -> impl IntoView {
    view! {
        <div class="background-2 rounded padded">
        <h2 class="h2"> Phone status</h2>
        {phone.map(|phone| {
            let active =phone.last_updated.signed_duration_since(Utc::now().naive_utc()) < Duration::minutes(5);
            Either::Left(view!{
            <span class:green={active} class:red={!active}>{format!("Ostatni kontakt: {}", phone.last_updated)}</span>
            <span>{format!("Sygnał: {}/100", phone.signal)}</span>
        })}).unwrap_or(Either::Right(view!{<span>Nie wykryto modemu</span>}))}
        </div>
        <div class="background-2 rounded padded flex-1">
            
        </div>
    }
}
