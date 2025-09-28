use leptos::prelude::*;

#[component]
pub fn Loader(children: ChildrenFn) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <div class="loading"></div> }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error rounded-3 flex-1 flex justify-center align-center padded">
                        <ul class="flex-1 vertical">
                            {
                                let a = errors.get();
                                a.into_iter()
                                    .map(|(e_id, error)| {
                                        view! {
                                            <li class="error-item">{format!("{}: {}", e_id, error)}</li>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }
                        </ul>
                    </div>
                }
            }>{children()}</ErrorBoundary>
        </Suspense>
    }
}
