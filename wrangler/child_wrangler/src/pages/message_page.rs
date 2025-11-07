use dto::messages::ContactDto;
use leptos::{either::Either, prelude::*};
use leptos_router::components::Outlet;

use crate::{components::loader::Loader, services::messages::get_contacts};

#[component]
pub fn MessagePage() -> impl IntoView {
    let contacts = Resource::new(|| (), |_| async move { get_contacts().await });
    view! {
        <Loader>
            {move || Suspend::new(async move {
                let contacts = contacts.await?;
                Ok::<_, ServerFnError>(view! { <InnerMessagePage contacts /> })
            })}
        </Loader>
    }
}

#[component]
pub fn InnerMessagePage(contacts: Vec<ContactDto>) -> impl IntoView {
    let (search, set_search) = signal(String::new());

    let searched_contacts = move || {
        let mut contacts = contacts.clone();
        let search = search();
        contacts.sort_by_key(|c| match c {
            ContactDto::Unknown(_) => format!("Nieznany"),
            ContactDto::GuardianWithPhone(guardian) => guardian.fullname.clone(),
        });
        contacts
            .into_iter()
            .filter(|c| match c {
                ContactDto::Unknown(p) => p.contains(&search),
                ContactDto::GuardianWithPhone(guardian) => {
                    guardian
                        .fullname
                        .to_lowercase()
                        .contains(&search.to_lowercase())
                        || guardian
                            .phone
                            .clone()
                            .map(|p| p.contains(&search))
                            .unwrap_or(false)
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="flex-1 flex flex-row gap-2 p-0.5">
            <div class="flex flex-col gap-2 p-0.5">
                <div class="flex-1 overflow-auto">
                    <ul class="flex-1 flex flex-col md:w-72 gap-0.5">
                        {move || {
                            searched_contacts()
                                .into_iter()
                                .map(|g| {
                                    view! {
                                        <li class="bg-gray-800 rounded-md overflow-hidden text-left w-full flex">
                                            {match g {
                                                ContactDto::Unknown(u) => {
                                                    Either::Left(
                                                        view! {
                                                            <a
                                                                class="md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600 flex-1 p-2 flex place-content-between"
                                                                href=format!("/messages/unknown/{}", u)
                                                            >
                                                                <span class="text-red-800">Nieznany</span>
                                                                <span class="">{format!("{}", u)}</span>
                                                            </a>
                                                        },
                                                    )
                                                }
                                                ContactDto::GuardianWithPhone(guardian) => {
                                                    Either::Right(
                                                        view! {
                                                            <a
                                                                class="md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600 flex-1 p-2 flex place-content-between"
                                                                href=format!("/messages/guardian/{}", guardian.id)
                                                            >
                                                                <span>{format!("{}", guardian.fullname)}</span>
                                                                {match guardian.phone {
                                                                    Some(phone) => {
                                                                        Either::Left(
                                                                            view! { <span class="">{format!("{}", phone)}</span> },
                                                                        )
                                                                    }
                                                                    None => {
                                                                        Either::Right(
                                                                            view! { <span class="text-red-800">Brak numeru</span> },
                                                                        )
                                                                    }
                                                                }}
                                                            </a>
                                                        },
                                                    )
                                                }
                                            }}
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </ul>
                </div>
                <input
                    autocomplete="off"
                    class="input"
                    bind:value=(search, set_search)
                    placeholder="Szukaj"
                />
            </div>
            <Outlet />
        </div>
    }
}
