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
        <div class="horizontal flex-1 gap overflow-hidden">
            <div class="vertical gap w-20 rounded min-w-10">
                <div class="vertical padded background-2 flex-1 rounded min-w-10 overflow-hidden">
                    <div class="overflow-auto">
                        <ul class="flex-1 vertical gap-2">
                            {move || {
                                searched_contacts()
                                    .iter()
                                    .map(|g| {
                                        match g {
                                            ContactDto::Unknown(u) => {
                                                Either::Left(
                                                    view! {
                                                        <li class="rounded vertical text-left">
                                                            <a
                                                                class="interactive horizontal padded-2 flex-1 rounded space-between"
                                                                href=format!("/messages/unknown/{}", u)
                                                            >
                                                                <span>Nieznany</span>
                                                                <span>{format!("{}", u)}</span>
                                                            </a>
                                                        </li>
                                                    },
                                                )
                                            }
                                            ContactDto::GuardianWithPhone(guardian) => {
                                                Either::Right(
                                                    view! {
                                                        <li class="rounded vertical">
                                                            <a
                                                                class="interactive horizontal padded-2 flex-1 rounded flex-start"
                                                                href=format!("/messages/guardian/{}", guardian.id)
                                                            >
                                                                {format!("{}", guardian.fullname)}
                                                            </a>
                                                        </li>
                                                    },
                                                )
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </ul>
                    </div>
                </div>
                <input
                    autocomplete="off"
                    class="rounded padded"
                    bind:value=(search, set_search)
                    placeholder="Szukaj"
                />
            </div>
            <Outlet />
        </div>
    }
}
