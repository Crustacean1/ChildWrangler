use std::collections::HashMap;

use dto::messages::{CancellationRequest, MessageProcessing, RequestError, Student, Token};
use leptos::{either::Either, prelude::*};
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    icons::refresh::RefreshIcon,
    services::messages::{get_message_processing_info, requeue_message},
};

#[component]
pub fn MessageDetailsModal(msg_id: i32) -> impl IntoView {
    let details = Resource::new(
        || (),
        move |_| async move { get_message_processing_info(msg_id).await },
    );

    view! {
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let details = details.await?;
                    Ok::<_, ServerFnError>(view! { <MessageDetailsModalInner msg_id details /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn MessageDetailsModalInner(
    msg_id: i32,
    details: HashMap<Uuid, Vec<MessageProcessing>>,
) -> impl IntoView {
    let snackbar = use_snackbar();
    let reprocess = Action::new(move |_: &()| async move {
        match requeue_message(msg_id).await {
            Ok(_) => snackbar.success("Wiadomość zostanie ponownie przetworzona"),
            Err(e) => snackbar.error("Nie udało się przetworzyć wiadomości ponownie", e),
        }
    });

    view! {
        <div class="vertical gap">
            <h2 class="h2 horizontal gap space-between">
                Szczegóły wiadomości <button class="interactive icon-button">
                    <RefreshIcon />
                </button>
            </h2>
            <div class="vertical gap-0">
                {details
                    .into_iter()
                    .map(|(id, details)| {
                        view! {
                            <div>{format!("Processing id: {}", id)}</div>
                            {details
                                .into_iter()
                                .map(|stage| match stage {
                                    MessageProcessing::Context(students) => {
                                        Either::Left(
                                            Either::Left(
                                                Either::Left(view! { <ContextInfo students /> }),
                                            ),
                                        )
                                    }
                                    MessageProcessing::Tokens(tokens) => {
                                        Either::Left(
                                            Either::Left(Either::Right(view! { <TokenInfo tokens /> })),
                                        )
                                    }
                                    MessageProcessing::Cancellation(request) => {
                                        Either::Left(
                                            Either::Right(
                                                Either::Left(view! { <CancellationInfo request /> }),
                                            ),
                                        )
                                    }
                                    MessageProcessing::StudentCancellation(cancellation) => {
                                        Either::Left(
                                            Either::Right(
                                                Either::Right(
                                                    view! { <StudentCancellation cancellation /> },
                                                ),
                                            ),
                                        )
                                    }
                                    MessageProcessing::RequestError(error) => {
                                        Either::Right(view! { <ComponentError error /> })
                                    }
                                })
                                .collect::<Vec<_>>()}
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn ContextInfo(students: Vec<Student>) -> impl IntoView {
    view! {
        <div class="content-green vertical gap-0">
            <span>Kontekst wiadomości</span>
            <ul class="vertical gap-0">
                {students
                    .into_iter()
                    .map(|student| {
                        view! {
                            <li class="vertical gap-0 content-blue">
                                <span>{format!("{} {}", student.name, student.surname)}</span>
                                <span>
                                    {format!(
                                        "Odmówienia przyjmowane do godziny: {}",
                                        student.grace_period.format("%H:%M:%S"),
                                    )}
                                </span>
                                <ul class="horizontal justify-center align-center gap flex-start">
                                    <li>Korzysta z posiłków:</li>
                                    {student
                                        .meals
                                        .into_iter()
                                        .map(|meal| {
                                            view! { <li class="pill">{format!("{}", meal.name)}</li> }
                                        })
                                        .collect::<Vec<_>>()}
                                </ul>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[component]
pub fn TokenInfo(tokens: Vec<Token>) -> impl IntoView {
    view! {
        <div class="content-green vertical gap-0">
            <span>Interpretacja wiadomości</span>
            <div class="horizontal gap">
                {tokens
                    .into_iter()
                    .map(|token| match token {
                        Token::Student(uuid) => {
                            Either::Left(
                                Either::Left(
                                    Either::Left(
                                        view! {
                                            <div class="pill outline-white">{format!("{}", uuid)}</div>
                                        },
                                    ),
                                ),
                            )
                        }
                        Token::Date(naive_date) => {
                            Either::Left(
                                Either::Left(
                                    Either::Right(
                                        view! {
                                            <div class="pill outline-white">
                                                {format!("{}", naive_date)}
                                            </div>
                                        },
                                    ),
                                ),
                            )
                        }
                        Token::Meal(uuid) => {
                            Either::Left(
                                Either::Right(
                                    Either::Left(
                                        view! {
                                            <div class="pill outline-white">{format!("{}", uuid)}</div>
                                        },
                                    ),
                                ),
                            )
                        }
                        Token::Unknown(i) => {
                            Either::Left(
                                Either::Right(
                                    Either::Right(
                                        view! {
                                            <div class="pill outline-red">{format!("{}", i)}</div>
                                        },
                                    ),
                                ),
                            )
                        }
                        Token::Ambiguous(i) => {
                            Either::Right(
                                view! { <div class="pill outline-red">{format!("{}", i)}</div> },
                            )
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn ComponentError(error: RequestError) -> impl IntoView {
    view! { <div class="content-red">{format!("{:?} ", error)}</div> }
}

#[component]
pub fn CancellationInfo(request: CancellationRequest) -> impl IntoView {
    view! {
        <div class="content-green vertical gap">
            {format!("Czas trwania: od {} do {}", request.since, request.until)} <div>
                <span>Uczniowie</span>
                {request
                    .students
                    .into_iter()
                    .map(|s| view! { <div class="pill">{format!("{}", s)}</div> })
                    .collect::<Vec<_>>()}
            </div> <div>
                <span>Posiłki</span>
                {if request.meals.is_empty() {
                    Either::Left(view! { <div>Wszystkie</div> })
                } else {
                    Either::Right(
                        request
                            .meals
                            .into_iter()
                            .map(|meal| view! { <div>{format!("{}", meal)}</div> })
                            .collect::<Vec<_>>(),
                    )
                }}
            </div>
        </div>
    }
}

#[component]
pub fn StudentCancellation(cancellation: Vec<dto::messages::StudentCancellation>) -> impl IntoView {
    view! {
        <div class="content-green gap vertical">
            <span>Odwołania</span>
            {cancellation
                .into_iter()
                .map(|cancellation| {
                    view! {
                        <div class="content-blue vertical gap">
                            <span>{format!("Id {}", cancellation.id)}</span>
                            <span>{format!("Meals: {:?}", cancellation.meals)}</span>
                            <span>
                                {format!("Od: {} do: {}", cancellation.since, cancellation.until)}
                            </span>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
