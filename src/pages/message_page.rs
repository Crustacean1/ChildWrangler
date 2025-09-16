use leptos::{either::Either, prelude::*};
use leptos_router::components::Outlet;

use crate::{
    dtos::{
        catering::{GuardianDetailDto, GuardianDto},
        messages::ContactDto,
    },
    services::{messages::get_contacts, student::get_guardians},
};

#[component]
pub fn MessagePage() -> impl IntoView {
    let contacts = Resource::new(|| (), |_| async move { get_contacts().await });
    view! {
        <Suspense>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let contacts = contacts.await?;
                    Ok::<_, ServerFnError>(view! { <InnerMessagePage contacts /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn InnerMessagePage(contacts: Vec<ContactDto>) -> impl IntoView {
    let (search, set_search) = signal(String::new());

    let searched_contacts = move || {
        let mut contacts = contacts.clone();
        contacts.sort_by_key(|c| match c {
            ContactDto::Unknown(_) => format!("Nieznany"),
            ContactDto::GuardianWithPhone(guardian) => guardian.fullname.clone(),
        });
        contacts
    };

    view! {
        <div class="horizontal flex-1 gap">
            <div class="vertical background-2 gap padded w-20 rounded">
                <div class="scrollable">
                    <ul class="flex-1 vertical gap-2">
                        {move || searched_contacts()
                            .iter()
                            .map(|g| {
                                match g {
                                    ContactDto::Unknown(u) => {
                                        Either::Left(
                                            view! {
                                                <li class="rounded vertical text-left">
                                                    <a
                                                        class="interactive horizontal padded flex-1 rounded space-between"
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
                                                        class="interactive horizontal padded flex-1 rounded flex-start"
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
                            .collect::<Vec<_>>()}
                    </ul>
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
