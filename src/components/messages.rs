use leptos::{either::Either, prelude::*};

use crate::{
    components::snackbar::{use_snackbar, MsgType, SnackbarContext},
    dtos::messages::{Message, MessageType},
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
                            <div class="flex-1 background-2 vertical gap padded rounded">
                                <InnerMessages messages />
                                <div class="horizontal gap">
                                    <input
                                        class="padded rounded flex-1"
                                        autocomplete="off"
                                        bind:value=(msg, set_msg)
                                    />
                                    <button
                                        class="rounded padded interactive"
                                        disabled=send_msg.pending()
                                        on:click=move |_| {
                                            send_msg.dispatch(msg());
                                        }
                                    >
                                        Wyślij
                                    </button>
                                </div>
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
    view! {
        <ul class="flex-1">
            {if messages.is_empty() {
                Either::Left(view! { <li class="padded dashed rounded">Brak wiadomości</li> })
            } else {
                Either::Right(view! {})
            }}
            {messages
                .iter()
                .map(|message| {
                    view! {
                        <div
                            class="rounded padded fit-content background-3 vertical"
                            class:self-start=if let MessageType::Received(_) = message.msg_type {
                                true
                            } else {
                                false
                            }
                            class:self-end=message.msg_type == MessageType::Sent
                                || message.msg_type == MessageType::Pending
                        >
                            <span>{format!("{}", message.content)}</span>
                            <small class="self-start gray">
                                {format!("Odebrano: {}", message.sent.format("%H:%M:%S"))}
                            </small>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}
