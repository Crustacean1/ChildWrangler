use std::collections::HashMap;

use chrono::NaiveDate;
use dto::{
    attendance::{AttendanceHistoryDto, AttendanceHistoryItemDto, GetAttendanceHistoryDto},
    catering::MealDto,
    group::GroupDto,
    student::StudentDto,
};
use leptos::{either::Either, prelude::*};
use uuid::Uuid;

use crate::{
    components::{
        dropdown::Dropdown,
        general_provider::{GroupResource, MealResource, StudentResource},
    },
    services::attendance::get_attendance_history,
};

#[component]
pub fn MealHistoryModal(target: Uuid, date: NaiveDate) -> impl IntoView {
    let history = Resource::new(
        || (),
        move |_| async move { get_attendance_history(GetAttendanceHistoryDto { target, date }).await },
    );
    let students = expect_context::<StudentResource>();
    let groups = expect_context::<GroupResource>();
    let meals = expect_context::<MealResource>();

    view! {
        <h2 class="text-center text-lg">Lista wydarzeń</h2>
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {Suspend::new(async move {
                    let students = students.0.await?;
                    let groups = groups.0.await?;
                    let history = history.await?;
                    let meals = meals.0.await?;
                    Ok::<
                        _,
                        ServerFnError,
                    >(view! { <MealHistoryModalInner history students groups meals /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn MealHistoryModalInner(
    history: AttendanceHistoryDto,
    meals: HashMap<Uuid, MealDto>,
    students: HashMap<Uuid, StudentDto>,
    groups: HashMap<Uuid, GroupDto>,
) -> impl IntoView {
    view! {
        <ul class="before:content-[''] before:bg-gray-400/20 before:rounded-full before:min-w-[1px] before:-left-[9px] before:absolute before:h-full relative ml-2 flex flex-col gap-4">
            {history
                .events
                .into_iter()
                .map(|att| {
                    view! {
                        <li class="relative before:content-[''] before:bg-gray-400 before:rounded-full before:min-w-2 before:min-h-2 before:absolute before:-left-3 before:top-1 ">
                            <h4 class="text-xs text-gray-300">
                                {format!("{}", att.time.format("%Y %B %d %H:%M:%S"))}
                            </h4>
                            {match att.item {
                                dto::attendance::AttendanceItemDto::Cancellation(_, _, _) => {
                                    Either::Left(Either::Left(view! {}))
                                }
                                dto::attendance::AttendanceItemDto::Override(id, reason) => {
                                    Either::Left(
                                        Either::Right(
                                            view! {
                                                <div class="flex flex-col gap-2">
                                                    <h2 class="text-lg">
                                                        Nadpisano obecność dla
                                                        <span class="rounded-md p-1 bg-gray-400 outline outline-gray-300">
                                                            {groups
                                                                .get(&id)
                                                                .map(|g| g.name.clone())
                                                                .unwrap_or(
                                                                    students
                                                                        .get(&id)
                                                                        .map(|s| format!("{} {}", s.name, s.surname))
                                                                        .unwrap_or(format!("Nieznany obiekt")),
                                                                )}
                                                        </span>

                                                    </h2>
                                                    {att
                                                        .meals
                                                        .iter()
                                                        .map(|meal| {
                                                            view! {
                                                                <div
                                                                    class="w-fit p-1 outline-2 rounded-md flex gap-2"
                                                                    class:outline-red-800=!meal.1
                                                                    class:bg-red-600=!meal.1
                                                                    class:outline-green-800=meal.1
                                                                    class:bg-green-600=meal.1
                                                                >
                                                                    {meals
                                                                        .get(&meal.0)
                                                                        .map(|meal| format!("{}", meal.name))
                                                                        .unwrap_or(format!("Nieprawidłówy posiłek"))}
                                                                </div>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                    {if reason.is_empty() {
                                                        Either::Left(
                                                            view! {
                                                                <span class="text-xs text-gray-600">Nie podano powodu</span>
                                                            },
                                                        )
                                                    } else {
                                                        Either::Right(
                                                            view! {
                                                                <span class="text-sx text-gray-600">{reason}</span>
                                                            },
                                                        )
                                                    }}
                                                </div>
                                            },
                                        ),
                                    )
                                }
                                dto::attendance::AttendanceItemDto::Init => {
                                    Either::Right(
                                        view! { <h2 class="text-lg">Dodano catering</h2> },
                                    )
                                }
                            }}
                        </li>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}
