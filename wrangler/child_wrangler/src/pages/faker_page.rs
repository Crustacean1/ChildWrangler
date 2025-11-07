use leptos::logging::log;
use leptos::prelude::*;

#[component]
pub fn FakerPage() -> impl IntoView {
    view! {
        <div class="flex flex-col flex-1 gap-2">
            <label for="phone">Phone</label>
            <input id="phone" class="input" />
            <div class="flex flex-col bg-gray-900 min-h-72"></div>
            <form
                class="flex flex-row gap-2"
                on:submit=|e| {
                    e.prevent_default();
                    log!("Hello, I am submitting to You :)");
                }
            >
                <input id="msg" class="input" />
                <button class="bg-gray-800 btn">Send</button>
            </form>
        </div>
    }
}
