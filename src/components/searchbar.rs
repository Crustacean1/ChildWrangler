use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::{
    components::dropdown::Dropdown, dtos::group::SearchTerm, services::group::get_search_terms,
};

#[component]
pub fn Searchbar() -> impl IntoView {
    let terms = Resource::new(|| (), |_| async move { get_search_terms().await });

    view! {
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let terms = terms.await?;
                    Ok::<_, ServerFnError>(view! { <SearchbarInner terms /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn SearchbarInner(terms: Vec<SearchTerm>) -> impl IntoView {
    let on_select = move |item: Result<SearchTerm, String>| {
        if let Ok(item) = item {
            let navigate = use_navigate();
            navigate(&format!("/attendance/{}", item.id), Default::default());
        }
    };

    view! {
        <Dropdown
            name="global_search"
            options=move || terms.clone()
            key=|t| t.id
            filter=|h, n| n.name.to_lowercase().contains(&h.to_lowercase())
            on_select
            item_view=|item| {
                view! {
                    <div class="horizontal padded trail align-center">
                        {item.parent_name.map(|name| view! { <div>{format!("{}", name)}</div> })}
                        <div class="horizontal align-center">{format!("{}", item.name)}</div>
                    </div>
                }
            }
        />
    }
}
