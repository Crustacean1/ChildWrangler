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
    icons::phone::PhoneIcon,
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
        <ul class="pt-5 before:content-[''] before:bg-gray-400/20 before:rounded-full before:min-w-[1px] before:top-0 before:-left-[9px] before:absolute before:h-full relative ml-2 flex flex-col gap-4">
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
                                dto::attendance::AttendanceItemDto::Cancellation(
                                    msg_id,
                                    phone,
                                    reason,
                                ) => {
                                    Either::Left(
                                        Either::Left(
                                            view! {
                                                <div class="flex flex-col gap-2">
                                                    <h2 class="flex gap-2">
                                                    <span class="text-lg">
                                                        Odmówiono obecność
                                                    </span>
                                                        <span class="rounded-md flex align-center pl-2 pr-2 bg-violet-500/50 outline outline-violet-500 flex gap-2">
                                                            <PhoneIcon />
                                                            {format!("{}", phone)}
                                                        </span>
                                                    </h2>
                                                    <div class="flex flex-col gap-2">
                                                        <span class="rounded-md flex align-center p-1 pl-2 pr-2 bg-orange-500/50 outline outline-orange-500 flex gap-2">{format!("{}", reason)}</span>
                                                    </div>

                                                    <ul class="flex flex-row gap-2">
                                                        {att
                                                            .meals
                                                            .iter()
                                                            .map(|meal| {
                                                                view! {
                                                                    <li class="w-fit p-1 outline rounded-md gap-2 outline-yellow-800 bg-yellow-800/50">
                                                                        {meals
                                                                            .get(&meal.0)
                                                                            .map(|meal| format!("{}", meal.name))
                                                                            .unwrap_or(format!("Nieprawidłówy posiłek"))}
                                                                    </li>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                    </ul>
                                                </div>
                                            },
                                        ),
                                    )
                                }
                                dto::attendance::AttendanceItemDto::Override(id, reason) => {
                                    Either::Left(
                                        Either::Right(
                                            view! {
                                                <div class="flex flex-col gap-2">
                                                    <h2 class="text-lg">
                                                        Nadpisano obecność dla
                                                        <a
                                                            class="rounded-md p-1 bg-gray-600/50 outline outline-gray-600"
                                                            href=format!("/attendance/{}", id)
                                                        >
                                                            {groups
                                                                .get(&id)
                                                                .map(|g| g.name.clone())
                                                                .unwrap_or(
                                                                    students
                                                                        .get(&id)
                                                                        .map(|s| format!("{} {}", s.name, s.surname))
                                                                        .unwrap_or(format!("Nieznany obiekt")),
                                                                )}
                                                        </a>

                                                    </h2>

                                                    <ul class="flex flex-row gap-2">
                                                        {att
                                                            .meals
                                                            .iter()
                                                            .map(|meal| {
                                                                view! {
                                                                    <li
                                                                        class="w-fit p-1 outline-2 rounded-md gap-2"
                                                                        class:outline-red-800=!meal.1
                                                                        class:bg-red-600=!meal.1
                                                                        class:outline-green-800=meal.1
                                                                        class:bg-green-600=meal.1
                                                                    >
                                                                        {meals
                                                                            .get(&meal.0)
                                                                            .map(|meal| format!("{}", meal.name))
                                                                            .unwrap_or(format!("Nieprawidłówy posiłek"))}
                                                                    </li>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                    </ul>
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
