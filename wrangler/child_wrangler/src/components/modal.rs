use leptos::prelude::*;

#[component]
pub fn Modal(
    is_open: impl Fn() -> bool + Send + Sync + Copy + 'static,
    on_close: impl Fn() + Send + Sync + Copy + 'static,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div
            class="absolute-position"
            style:top="0"
            style:display=move || if is_open() { "flex" } else { "none" }
            style:left="0"
            style:width="100vw"
            style:height="100vh"
            style:justify-content="center"
            style:align-items="center"
            style:background-color="rgba(0,0,0,0.25)"
            on:click=move |_| on_close()
        >
            <div
                class="vertical pretty-background padded gap rounded"
                on:click=|e| e.stop_propagation()
            >
                <Show when=is_open>{children()}</Show>
            </div>
        </div>
    }
}
