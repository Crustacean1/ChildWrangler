use std::collections::{BTreeMap, HashMap};

use dto::messages::{Message, MessageType};
use leptos::{either::Either, logging::log, prelude::*};

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
                            <div class="flex-1 background-2 vertical gap padded rounded overflow-hidden">
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
    let (show_details, set_show_details) = signal(None::<i32>);

    let mut sorted_messages = BTreeMap::new();

    for message in messages {
        let day = sorted_messages.entry(message.sent.date()).or_insert(vec![]);
        day.push(message);
        day.sort_by_key(|m| m.sent);
    }

    view! {
        <div class="overflow-auto flex-1 flex flex-col-reverse gap-2 card">
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
                                view! {
                                    <div
                                        on:click=move |_| set_show_details(Some(message.id))
                                        class="rounded padded fit-content background-3 vertical"
                                        class:self-start=if let MessageType::Received(_) = message
                                            .msg_type
                                        {
                                            true
                                        } else {
                                            false
                                        }
                                        class:self-end=message.msg_type == MessageType::Sent
                                            || message.msg_type == MessageType::Pending
                                    >
                                        <span>{format!("{}", message.content)}</span>
                                        <small class="self-end gray">
                                            {format!("Odebrano: {}", message.sent.format("%H:%M:%S"))}
                                        </small>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()}
                        <div class="date-break">
                            <span>{format!("{}", day.format("%d-%m-%Y"))}</span>
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
