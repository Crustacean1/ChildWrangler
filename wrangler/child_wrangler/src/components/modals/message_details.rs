use std::collections::HashMap;

use dto::{
    catering::MealDto,
    messages::{
        AttendanceCancellation, CancellationRequest, CancellationResult, MessageProcessing,
        RequestError, Student, Token,
    },
    student::StudentDto,
};
use leptos::{either::Either, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        general_provider::{MealResource, StudentResource},
        snackbar::{use_snackbar, SnackbarContext},
    },
    icons::{
        calendar::CalendarIcon, meal::MealIcon, person::PersonIcon, question::QuestionIcon,
        refresh::RefreshIcon,
    },
    services::messages::{get_message_processing_info, requeue_message},
};

#[component]
pub fn MessageDetailsModal(msg_id: Uuid) -> impl IntoView {
    let details = Resource::new(
        || (),
        move |_| async move { get_message_processing_info(msg_id).await },
    );
    let students = expect_context::<StudentResource>().0;
    let meals = expect_context::<MealResource>().0;

    view! {
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let details = details.await?;
                    let students = students.await?;
                    let meals = meals.await?;
                    Ok::<
                        _,
                        ServerFnError,
                    >(view! { <MessageDetailsModalInner msg_id details students meals /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn MessageDetailsModalInner(
    msg_id: Uuid,
    details: HashMap<Uuid, Vec<MessageProcessing>>,
    students: HashMap<Uuid, StudentDto>,
    meals: HashMap<Uuid, MealDto>,
) -> impl IntoView {
    let snackbar = use_snackbar();
    let reprocess = Action::new(move |_: &()| async move {
        match requeue_message(msg_id).await {
            Ok(_) => snackbar.success("Wiadomość zostanie ponownie przetworzona"),
            Err(e) => snackbar.error("Nie udało się przetworzyć wiadomości ponownie", e),
        }
    });

    view! {
        <div class="flex flex-col gap-2">
            <div class="flex flex-col gap-2">
                {details
                    .into_iter()
                    .map(|(id, details)| {
                        view! {
                            <div>{format!("Processing id: {}", id)}</div>
                            {details
                                .into_iter()
                                .map(|stage| match stage {
                                    MessageProcessing::Init => {
                                        Either::Left(
                                            Either::Left(Either::Left(view! { <ContextInfo /> })),
                                        )
                                    }
                                    MessageProcessing::Tokens(tokens) => {
                                        Either::Left(
                                            Either::Left(
                                                Either::Right(
                                                    view! {
                                                        <TokenInfo
                                                            tokens
                                                            students=students.clone()
                                                            meals=meals.clone()
                                                        />
                                                    },
                                                ),
                                            ),
                                        )
                                    }
                                    MessageProcessing::Cancellation(request) => {
                                        Either::Left(
                                            Either::Right(
                                                Either::Left(
                                                    view! {
                                                        <CancellationInfo
                                                            request
                                                            students=students.clone()
                                                            meals=meals.clone()
                                                        />
                                                    },
                                                ),
                                            ),
                                        )
                                    }
                                    MessageProcessing::StudentCancellation(cancellation) => {
                                        Either::Left(
                                            Either::Right(
                                                Either::Right(
                                                    view! {
                                                        <StudentCancellation
                                                            cancellation
                                                            students=students.clone()
                                                            meals=meals.clone()
                                                        />
                                                    },
                                                ),
                                            ),
                                        )
                                    }
                                    MessageProcessing::RequestError(error) => {
                                        Either::Right(
                                            Either::Left(view! { <ComponentError error /> }),
                                        )
                                    }
                                    MessageProcessing::CancellationResult(result) => {
                                        Either::Right(
                                            Either::Right(view! { <CancellationResultView result /> }),
                                        )
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
pub fn CancellationResultView(result: Vec<CancellationResult>) -> impl IntoView {
    view! { <div>Cancellation result</div> }
}

#[component]
pub fn ContextInfo() -> impl IntoView {
    /*
    view! {
        <div class="flex flex-col gap-2 bg-green-500/15 outline outline-green-500/50 rounded-md p-2">
            <h2 class="text-center">Kontekst wiadomości</h2>
            <ul class="flex flex-row gap-2">
                {students
                    .into_iter()
                    .map(|student| {
                        view! {
                            <li class="flex flex-col gap-1 bg-blue-500/15 outline outline-blue-500/50 p-2 rounded-md">
                                <span>
                                    {format!("Uczeń: {} {}", student.name, student.surname)}
                                </span>
                                <span>
                                    {format!(
                                        "Odmówienia do: {}",
                                        student.grace_period.format("%H:%M:%S"),
                                    )}
                                </span>
                                <ul class="flex flex-row gap-2">
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
    }*/
    view! { <div>Context goes here</div> }
}

#[component]
pub fn TokenInfo(
    tokens: Vec<Token>,
    students: HashMap<Uuid, StudentDto>,
    meals: HashMap<Uuid, MealDto>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 bg-green-500/15 outline outline-green-500/50 rounded-md p-2">
            <h2 class="text-center">Interpretacja wiadomości</h2>
            <div class="flex flex-row gap-2">
                {tokens
                    .into_iter()
                    .map(|token| match token {
                        Token::Student(uuid) => {
                            Either::Left(
                                Either::Left(
                                    Either::Left(
                                        view! {
                                            <div class=" flex p-1 pl-2 pr-2 gap-1 rounded-full outline-blue-500/50 outline bg-blue-500/15">
                                                <PersonIcon />
                                                {students
                                                    .get(&uuid)
                                                    .map(|s| format!("{} {}", s.name, s.surname))}
                                            </div>
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
                                            <div class="flex p-1 pl-2 pr-2  gap-1 rounded-full outline-violet-500/50 outline bg-violet-500/15">
                                                <CalendarIcon />
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
                                            <div class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-purple-500/50 outline bg-purple-500/15">
                                                <MealIcon />
                                                {meals.get(&uuid).map(|m| format!("{}", m.name))}
                                            </div>
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
                                            <div class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-red-500/50 outline bg-red-500/15">
                                                <QuestionIcon />
                                                {format!("{}", i)}
                                            </div>
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
    view! {
        <div class="p-2 rounded-md bg-red-500/15 outline outline-red-500/50">
            {format!("{:?} ", error)}
        </div>
    }
}

#[component]
pub fn CancellationInfo(
    request: CancellationRequest,
    students: HashMap<Uuid, StudentDto>,
    meals: HashMap<Uuid, MealDto>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 bg-green-500/15 outline outline-green-500/50 rounded-md p-2">
            <h2 class="text-center">Czas trwania</h2>
            {format!("{} - {}", request.since, request.until)}
            <div>
                <h2 class="text-center">Uczniowie</h2>
                <div class="flex gap-2">
                    {request
                        .students
                        .into_iter()
                        .map(|s| {
                            view! {
                                <div class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-blue-500/50 outline bg-blue-500/15">
                                    <PersonIcon />
                                    {students.get(&s).map(|s| format!("{} {}", s.name, s.surname))}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
            <div>
                <h2 class="text-center">Posiłki</h2>
                <div class="flex gap-2">
                    {if request.meals.is_empty() {
                        Either::Left(view! { <div>Wszystkie</div> })
                    } else {
                        Either::Right(
                            request
                                .meals
                                .into_iter()
                                .map(|meal| {
                                    view! {
                                        <div class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-violet-500/50 outline bg-violet-500/15">
                                            <MealIcon />
                                            {meals.get(&meal).map(|m| format!("{}", m.name))}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>(),
                        )
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StudentCancellation(
    cancellation: AttendanceCancellation,
    students: HashMap<Uuid, StudentDto>,
    meals: HashMap<Uuid, MealDto>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 bg-green-500/15 outline outline-green-500/50 rounded-md p-2">
            <span>Odwołania</span>
            {cancellation
                .students
                .into_iter()
                .map(|cancellation| {
                    view! {
                        <div class="flex flex-row gap-2">
                            <span class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-blue-500/50 outline bg-blue-500/15">
                                <PersonIcon />
                                {students
                                    .get(&cancellation.id)
                                    .map(|s| format!("{} {}", s.name, s.surname))}
                            </span>
                            {cancellation
                                .meals
                                .into_iter()
                                .map(|id| {
                                    view! {
                                        <div class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-violet-500/50 outline bg-violet-500/15">
                                            <MealIcon />
                                            {meals.get(&id).map(|meal| format!("{}", meal.name))}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                            <span class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-pink-500/50 outline bg-pink-500/15">
                                {format!("Od: {}", cancellation.since)}
                            </span>
                            <span class="flex p-1 pl-2 pr-2 gap-1 rounded-full outline-pink-500/50 outline bg-pink-500/15">
                                {format!("Do: {}", cancellation.until)}
                            </span>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
