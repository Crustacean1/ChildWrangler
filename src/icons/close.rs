use leptos::prelude::*;

#[component]
pub fn CloseIcon() -> impl IntoView {
    view! {
        <svg
            // on:click=move |_| set_meals(meals().into_iter().filter(|m| *m != meal2).collect())
            xmlns="http://www.w3.org/2000/svg"
            height="24px"
            viewBox="0 -960 960 960"
            width="24px"
            fill="#ff0000"
        >
            <path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z" />
        </svg>
    }
}
