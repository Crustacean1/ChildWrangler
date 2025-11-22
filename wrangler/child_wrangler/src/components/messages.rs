use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDateTime;
use dto::messages::{Message, PendingMessage, ReceivedMessage, SentMessage};
use leptos::{either::Either, logging::log, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        modal::Modal,
        modals::message_details::MessageDetailsModal,
        snackbar::{use_snackbar, SnackbarContext},
    },
    services::messages::{get_messages, send_message},
};

#[component]
pub fn Messages(phone: String) -> impl IntoView {
    let snackbar = use_snackbar();

    let send_msg = Action::new({
        let phone = phone.clone();
        move |content: &String| {
            let content = content.clone();
            let phone = phone.clone();
            async move {
                match send_message(phone, content).await {
                    Ok(_) => snackbar.success("Wysłano wiadomość"),
                    Err(e) => snackbar.error("Nie udało się wysłać wiadomości", e),
                }
            }
        }
    });

    let messages = Resource::new(
        move || phone.clone(),
        |phone| async move { get_messages(phone).await },
    );
    let (msg, set_msg) = signal(String::new());

    let sending_disabled = move || send_msg.pending()() || msg().trim().is_empty();

    view! {
        <Transition>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let messages = messages.await?;
                    Ok::<
                        _,
                        ServerFnError,
                    >(
                        view! {
                            <div class="flex-1 background-2 vertical gap padded rounded overflow-auto">
                                <InnerMessages messages />
                            </div>
                            <div class="flex flex-row gap-2">
                                <input
                                    class="input flex-1"
                                    autocomplete="off"
                                    bind:value=(msg, set_msg)
                                />
                                <button
                                    class="bg-gray-900 btn"
                                    disabled=sending_disabled
                                    on:click=move |_| {
                                        send_msg.dispatch(msg());
                                        set_msg(String::new());
                                    }
                                >
                                    Wyślij
                                </button>
                            </div>
                        },
                    )
                })}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn InnerMessages(messages: Vec<Message>) -> impl IntoView {
    let (show_details, set_show_details) = signal(None::<Uuid>);

    let mut sorted_messages = BTreeMap::new();

    for message in messages {
        let day = sorted_messages
            .entry(message.metadata().inserted.date())
            .or_insert(vec![]);
        day.push(message);
    }

    for (day, messages) in sorted_messages.iter_mut() {
        messages.sort_by_key(|m| match m {
            Message::Sent(sent_message) => sent_message.metadata.inserted,
            Message::Received(received_message) => received_message.received,
            Message::Pending(pending_message) => pending_message.metadata.inserted,
        });
    }

    view! {
        <div class="overflow-auto flex-1 gap-2 flex flex-col-reverse">
            {if sorted_messages.is_empty() {
                Either::Left(view! { <li class="padded dashed rounded">Brak wiadomości</li> })
            } else {
                Either::Right(view! {})
            }}
            {sorted_messages
                .into_iter()
                .rev()
                .map(|(day, messages)| {
                    view! {
                        {messages
                            .into_iter()
                            .rev()
                            .map(|message| {
                                match message {
                                    Message::Received(message) => {
                                        Either::Left(
                                            Either::Left(
                                                view! {
                                                    <ReceivedMessageView
                                                        on_click=move |id| set_show_details(Some(id))
                                                        message
                                                    />
                                                },
                                            ),
                                        )
                                    }
                                    Message::Sent(message) => {
                                        Either::Left(
                                            Either::Right(view! { <SentMessageView message /> }),
                                        )
                                    }
                                    Message::Pending(message) => {
                                        Either::Right(view! { <PendingMessageView message /> })
                                    }
                                }
                            })
                            .collect::<Vec<_>>()}
                        <div class="text-center before:w-full before:absolute before:top-[50%] before:-z-1 before:left-0 before:rounded-full before:bg-gray-300/25 before:h-0.5 before:content-[''] relative">
                            <span class="z-index-2 bg-gray-950 p-1">
                                {format!("{}", day.format("%d %B %Y"))}
                            </span>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
        <Modal
            is_open=move || show_details().is_some()
            on_close=move || {
                set_show_details(None);
            }
        >
            {move || {
                show_details()
                    .map(|msg_id| {
                        view! { <MessageDetailsModal msg_id /> }
                    })
            }}
        </Modal>
    }
}

#[component]
pub fn PendingMessageView(message: PendingMessage) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1 w-fit self-end">
            <div class="card row row-col p-2 w-fit">
                <span>{format!("{}", message.data.content)}</span>
            </div>
            <small class="self-end gray">
                {format!("Zakolejkowano: {}", message.metadata.inserted.format("%H:%M:%S"))}
            </small>
        </div>
    }
}

#[component]
pub fn SentMessageView(message: SentMessage) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1 w-fit self-end w-fit">
            <div on:click=move |_| {} class="card row row-col p-2 ">
                <span>{format!("{}", message.data.content)}</span>
            </div>
            <small class="self-end gray flex flex-row gap-2">
                <span>
                    {format!("Zakolejkowano: {}", message.metadata.inserted.format("%H:%M:%S"))}
                </span>
                <span>{format!("Wysłano: {}", message.sent.format("%H:%M:%S"))}</span>
            </small>
        </div>
    }
}

#[component]
pub fn ReceivedMessageView(
    message: ReceivedMessage,
    on_click: impl Fn(Uuid) + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1 w-fit self-start ">
            <div on:click=move |_| on_click(message.metadata.id) class="card row row-col p-2 ">
                <span>{format!("{}", message.data.content)}</span>
            </div>
            <small class="self-end gray flex gap-2">
                <span>{format!("Wysłano: {}", message.received.format("%H:%M:%S"))}</span>
                <span>
                    {format!("Otrzymano: {}", message.metadata.inserted.format("%H:%M:%S"))}
                </span>
            </small>
        </div>
    }
}
