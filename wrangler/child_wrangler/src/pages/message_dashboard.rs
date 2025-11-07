use chrono::{Duration, TimeDelta, Utc};
use dto::messages::{GeneralMessageDto, PhoneStatusDto};
use leptos::{either::Either, prelude::*};

use crate::{
    components::loader::Loader,
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
pub fn MessageView(message: GeneralMessageDto) -> impl IntoView {
    match message.msg_type {
        dto::messages::MessageType::Received(_) => Either::Left(Either::Left(view! {
            <div class="card p-2 self-start">
                <div class="padded">{format!("Od: {}", message.sender)}</div>
                <span class="spacer"></span>
                <div class="background-4 padded">{message.content}</div>
                <span class="spacer"></span>
                <div class="grid-2 gap padded">
                    <small class="gray">Wysłano</small>
                    <small class="gray">{format!("{:?}", message.sent)}</small>
                    <small class="gray">Otrzymano</small>
                    <small class="gray">{format!("{}", message.inserted)}</small>
                </div>
            </div>
            <div></div>
        })),
        dto::messages::MessageType::Pending => Either::Left(Either::Right(view! {
            <div class="card self-end p-2">{message.content}</div>
            <div></div>
        })),
        dto::messages::MessageType::Sent => Either::Right(view! {
            <div></div>
            <div class="card self-end p-2">
                <div class="padded">{format!("Do: {}", message.sender)}</div>
                <span class="spacer"></span>
                <div class="background-4 padded">{format!("{}", message.content)}</div>
                <span class="spacer"></span>
                <div class="grid-2 gap padded">
                    <small class="gray">Zakolejkowano</small>
                    <small class="gray">{format!("{:?}", message.sent)}</small>
                    <small class="gray">Wysłano</small>
                    <small class="gray">{format!("{}", message.inserted)}</small>
                </div>
            </div>
        }),
    }
}

#[component]
pub fn MessageDashboardInner(
    phone: Option<PhoneStatusDto>,
    mut messages: Vec<GeneralMessageDto>,
) -> impl IntoView {
    messages.sort_by_key(|m| m.inserted);

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
        <div class="grid grid-cols-3 gap-2">
            <div></div>
            <div style="grid-row: 1/9999; grid-column:2/3"></div>
            <div></div>
            {messages
                .into_iter()
                .rev()
                .map(|message| view! { <MessageView message /> })
                .collect::<Vec<_>>()}
        </div>
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
