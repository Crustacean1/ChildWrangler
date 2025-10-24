use leptos::prelude::*;

#[component]
pub fn ListIcon() -> impl IntoView {
    view! {
        <svg
            width="24px"
            height="24px"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M11 5H21M11 12H21M11 19H21"
                stroke="#FFFFFF"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
            />
            <rect
                height="4"
                rx="1"
                stroke="#FFFFFF"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                width="4"
                x="3"
                y="3"
            />
            <rect
                height="4"
                rx="1"
                stroke="#FFFFFF"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                width="4"
                x="3"
                y="10"
            />
            <rect
                height="4"
                rx="1"
                stroke="#FFFFFF"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                width="4"
                x="3"
                y="17"
            />
        </svg>
    }
}
