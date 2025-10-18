use leptos::prelude::*;

#[component]
pub fn Modal(
    is_open: impl Fn() -> bool + Send + Sync + Copy + 'static,
    on_close: impl Fn() + Send + Sync + Copy + 'static,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div
            class="flex absolute top-0 left-0 w-full h-full justify-center items-center backdrop-blur-xs "
            class:hidden=move || !is_open()
            on:click=move |_| on_close()
        >
            <div
                class="flex flex-col bg-gray-800 p-2 rounded-xl outline outline-white/15"
                on:click=|e| e.stop_propagation()
            >
                <Show when=is_open>{children()}</Show>
            </div>
        </div>
    }
}
