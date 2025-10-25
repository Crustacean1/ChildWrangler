use chrono::NaiveDate;
use dto::attendance::{AttendanceHistoryDto, AttendanceHistoryItemDto, GetAttendanceHistoryDto};
use leptos::{either::Either, prelude::*};
use uuid::Uuid;

use crate::{components::dropdown::Dropdown, services::attendance::get_attendance_history};

#[component]
pub fn MealHistoryModal(target: Uuid, date: NaiveDate) -> impl IntoView {
    let history = Resource::new(
        || (),
        move |_| async move { get_attendance_history(GetAttendanceHistoryDto { target, date }).await },
    );

    view! {
        <h2 class="text-center text-lg">Lista wydarzeń</h2>
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {Suspend::new(async move {
                    let history = history.await?;
                    Ok::<_, ServerFnError>(view! { <MealHistoryModalInner history /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn MealHistoryModalInner(history: AttendanceHistoryDto) -> impl IntoView {
    view! {
        <ul class="before:content-[''] before:bg-gray-400/20 before:rounded-full before:min-w-[1px] before:-left-[9px] before:absolute before:h-full relative ml-2 flex flex-col gap-2">
            {history
                .events
                .into_iter()
                .map(|att| {
                    view! {
                        <li class="relative before:content-[''] before:bg-gray-400 before:rounded-full before:min-w-2 before:min-h-2 before:absolute before:-left-3 before:top-1 ">
                            <h4 class="text-xs text-gray-300">{format!("{}", att.time.format("%Y %B %d %H:%M:%S"))}</h4>
                            {match att.item {
                                dto::attendance::AttendanceItemDto::Cancellation(_, _, _) => Either::Left(Either::Left(view!{})),
                                dto::attendance::AttendanceItemDto::Override(id, reason) => Either::Left(Either::Right(view!{
                                        <h2 class="text-lg">Nadpisano obecność w grupie</h2>
                                        <div>{format!("{}", id)}</div>
                                        {
                                        if reason.is_empty() {
                                            Either::Left(view!{
                                                <span class="text-xs text-gray-600" > Nie podano powodu</span>
                                            })
                                        }else{
                                            Either::Right(view!{
                                                <span class="text-sx text-gray-600">{reason}</span>
                                            })
                                        }
                                    }
                                {
                                    
                                }
                                        <div class="outline rounded-lg bg-red-500/20 outline-red-700 p-0.5 w-fit">śniadanie</div>
                                })),
                                dto::attendance::AttendanceItemDto::Init => Either::Right(view!{})
                            }}
                        </li>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}
