use chrono::{Duration, TimeDelta, Utc};
use dto::messages::{Message, PhoneStatusDto};
use leptos::{either::Either, prelude::*};

use crate::{
    components::{loader::Loader, messages::InnerMessages},
    services::messages::{get_latest_messages, get_phone_status},
};

#[component]
pub fn MessageDashboard() -> impl IntoView {
    let phone = Resource::new(|| (), move |_| async move { get_phone_status().await });
    let messages = Resource::new(
        || (),
        move |_| async move { get_latest_messages(TimeDelta::days(10)).await },
    );

    view! {
        <div class="overflow-hidden flex-1 flex flex-col gap-2">
            <Loader>
                {move || Suspend::new(async move {
                    let phone = phone.await?;
                    let messages = messages.await?;
                    Ok::<_, ServerFnError>(view! { <MessageDashboardInner phone messages /> })
                })}
            </Loader>
        </div>
    }
}

#[component]
pub fn MessageDashboardInner(
    phone: Option<PhoneStatusDto>,
    messages: Vec<Message>,
) -> impl IntoView {
    view! {
        <div class="card">
            <h2 class="h2">Phone status</h2>
            {phone
                .map(|phone| {
                    let active = Utc::now().naive_utc().signed_duration_since(phone.last_updated)
                        < Duration::minutes(5);
                    let signal_color_g = (255.0 * phone.signal as f32 / 100.0) as i32;
                    let signal_color_r = (255.0 - signal_color_g as f32) as i32;
                    Either::Left(

                        view! {
                            <div class="horizontal gap">
                                <span>Ostatni kontakt</span>
                                <span class:green=active class:red=!active>
                                    {format!("{}", phone.last_updated)}
                                </span>
                                <span>Sygnał</span>
                                <span style:color=format!(
                                    "rgb({},{},0)",
                                    signal_color_r,
                                    signal_color_g,
                                )>{format!("{}", phone.signal)}</span>
                                <span>/</span>
                                <span>100</span>
                            </div>
                        },
                    )
                })
                .unwrap_or(Either::Right(view! { <span>Nie wykryto modemu</span> }))}
        </div>
        <InnerMessages messages />
    }
}

#[component]
pub fn IncomingMessage() -> impl IntoView {
    view! {}
}

#[component]
pub fn PendingMessage() -> impl IntoView {
    view! {}
}

#[component]
pub fn OutgoingMessage() -> impl IntoView {
    view! {}
}
