use leptos::prelude::*;

#[component]
pub fn Searchbar() -> impl IntoView {
    view! { <input placeholder="Szukaj" class="flex-1 rounded padded background-3" /> }
}
