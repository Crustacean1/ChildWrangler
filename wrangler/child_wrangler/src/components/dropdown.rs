use leptos::either::Either;
use leptos::logging::log;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::HtmlLiElement;
use leptos::{prelude::*, tachys::renderer::dom::Node};
use uuid::Uuid;

#[component]
pub fn Dropdown<T: Send + Sync + Clone + 'static, R>(
    name: &'static str,
    options: impl Fn() -> Vec<T> + Send + Sync + 'static,
    key: impl Fn(&T) -> Uuid + Clone + Copy + Send + Sync + 'static,
    filter: impl Fn(&str, &T) -> bool + Send + Sync + Copy + 'static,
    on_select: impl Fn(Result<T, String>) -> Option<String> + Send + Sync + Copy + 'static,
    item_view: impl Fn(T) -> R + Send + Sync + Copy + 'static,
) -> impl IntoView
where
    R: IntoView + 'static,
{
    //let on_select = |_|{};
    let input_ref = NodeRef::new();
    let list_ref = NodeRef::new();

    let (active, set_active) = signal(false);

    let (input_value, set_input_value) = signal(String::new());

    let filtered_options = Signal::derive(move || {
        options()
            .into_iter()
            .filter(|item| filter(&input_value(), item))
            .collect::<Vec<_>>()
    });

    view! {
        <div
            class="vertical relative flex-1"
            on:focusout=move |_| { set_active(false) }
            on:focusin=move |_| { set_active(true) }
            on:keydown=move |e| {
                e.stop_propagation();
                if e.key_code() == 40 {
                    e.prevent_default();
                    let active_element = window().document().and_then(|doc| doc.active_element());
                    let active_node: Option<Node> = active_element.clone().map(|e| e.into());
                    if input_ref
                        .get()
                        .map(|input| input.is_same_node(active_node.as_ref()))
                        .unwrap_or(false)
                    {
                        std::iter::successors(
                                list_ref.get().and_then(|node| node.first_element_child()),
                                |x| { x.next_element_sibling() },
                            )
                            .next()
                            .map(|next| {
                                next.dyn_ref::<HtmlLiElement>()
                                    .map(|e| {
                                        //e.scroll_into_view_with_bool(true);
                                        e.focus().ok()
                                    })
                            });
                    } else {
                        std::iter::successors(
                                active_element.and_then(|e| e.next_element_sibling()),
                                |x| { x.next_element_sibling() },
                            )
                            .next()
                            .map(|next| {
                                next.dyn_ref::<HtmlLiElement>()
                                    .map(|e| {
                                        //e.scroll_into_view_with_bool(true);
                                        e.focus().ok()
                                    })
                            });
                    }
                }
                if e.key_code() == 38 {
                    e.prevent_default();
                    let active_element = window().document().and_then(|doc| doc.active_element());
                    std::iter::successors(
                            active_element.and_then(|e| e.previous_element_sibling()),
                            |x| { x.previous_element_sibling() },
                        )
                        .next()
                        .map(|next| {
                            next.dyn_ref::<HtmlLiElement>()
                                .map(|e| {
                                    //e.scroll_into_view_with_bool(true);
                                    e.focus().ok()
                                })
                        });
                }
            }
        >
            <input
                id=name
                class="input flex-1 w-full"
                node_ref=input_ref
                bind:value=(input_value, set_input_value)
                autocomplete="off"
                on:keydown=move |e| {
                    set_active(true);
                    if e.key_code() == 13 {
                        e.prevent_default();
                        e.stop_propagation();
                        set_active(false);
                        if let Some(value) = on_select(Err(input_value())) {
                            set_input_value(String::new());
                        }
                    }
                }
            />
            {move || {
                if filtered_options().len() < 100 {
                    Either::Left(
                        view! {
                            <ul
                                class="bg-gray-700 max-h-48 rounded-md w-full flex flex-col overflow-auto absolute bot-0 z-2"
                                style:display=move || if active() { "flex" } else { "none" }
                                role="listbox"
                                node_ref=list_ref
                            >
                                <For
                                    each=filtered_options
                                    key
                                    children=move |item| {
                                        let item = Signal::derive(move || item.clone());
                                        view! {
                                            <li
                                                tabindex="-1"
                                                role="option"
                                                class="md:hover:bg-gray-600 md:active:bg-gray-700 focus:bg-gray-500 outline-none"
                                                on:mouseover=move |e| {
                                                    e.current_target()
                                                        .and_then(|e| {
                                                            e.dyn_ref::<HtmlLiElement>().map(|i| { i.focus() })
                                                        });
                                                }
                                                on:mousedown={
                                                    let on_select = on_select.clone();
                                                    move |e| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        if let Some(value) = on_select(Ok(item())) {
                                                            set_input_value(value);
                                                        }
                                                        if let Some(doc) = window().document() {
                                                            if let Some(e) = doc.active_element() {
                                                                e.dyn_ref::<HtmlLiElement>().map(|i| { i.blur().ok() });
                                                            }
                                                        }
                                                    }
                                                }
                                                on:keydown=move |e| {
                                                    if e.key_code() != 40 && e.key_code() != 38 {
                                                        input_ref.get().map(|input| input.focus());
                                                    }
                                                    if e.key_code() == 13 {
                                                        if let Some(value) = on_select(Ok(item())) {
                                                            set_input_value(value);
                                                        }
                                                        set_active(false);
                                                    }
                                                }
                                            >
                                                {move || item_view(item())}
                                            </li>
                                        }
                                    }
                                />
                            </ul>
                        },
                    )
                } else {
                    Either::Right(view! {})
                }
            }}
        </div>
    }
}
