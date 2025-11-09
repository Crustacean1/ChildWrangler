use leptos::logging::log;
use leptos::prelude::*;

use crate::services::messages::simulate_message;

#[component]
pub fn FakerPage() -> impl IntoView {
    let (phone, set_phone) = signal(String::new());
    let (content, set_content) = signal(String::new());

    let send_action = Action::new(move |_: &()| async move {
        simulate_message(phone(), content()).await.unwrap();
        set_content(String::new());
    });

    view! {
        <div class="flex flex-col flex-1 gap-2">
            <label for="phone">Phone</label>
            <input id="phone" class="input" bind:value=(phone, set_phone) />
            <div class="flex flex-col bg-gray-900 min-h-72"></div>
            <form
                class="flex flex-row gap-2"
                on:submit=|e| {
                    e.prevent_default();
                    log!("Hello, I am submitting to You :)");
                }
            >
                <input id="msg" class="input" bind:value=(content, set_content) />
                <button
                    class="bg-gray-800 btn"
                    disabled=send_action.pending()
                    on:click=move |_| {
                        send_action.dispatch(());
                    }
                >
                    Send
                </button>
            </form>
        </div>
    }
}
