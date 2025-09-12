use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::HtmlLiElement;
use leptos::{prelude::*, tachys::renderer::dom::Node};
use uuid::Uuid;

#[component]
pub fn Dropdown<T: Send + Sync + Clone + 'static, R>(
    name: &'static str,
    options: impl Fn() -> Vec<T> + Send + Sync + 'static,
    key: impl Fn(&T) -> Uuid + Clone + Send + Sync + 'static,
    filter: impl Fn(&str, &T) -> bool + Send + Sync + Copy + 'static,
    on_select: impl Fn(Result<T, String>) + Send + Sync + Copy + 'static,
    item_view: impl Fn(T) -> R + Send + Sync + Copy + 'static,
) -> impl IntoView
where
    R: IntoView + 'static,
{
    //let on_select = |_|{};
    let input_ref = NodeRef::new();
    let list_ref = NodeRef::new();

    let (active, set_active) = signal(false);
    let (value, set_value) = signal(None::<T>);

    let (input_value, set_input_value) = signal(String::new());

    let filtered_options = Signal::derive(move || {
        options()
            .into_iter()
            .filter(|item| filter(&input_value(), item))
            .collect::<Vec<_>>()
    });

    view! {
        <div
            class="vertical relative"
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
                            .filter(|x| {
                                x.dyn_ref::<HtmlLiElement>()
                                    .map(|e| e.class_list().contains("active"))
                                    .unwrap_or(false)
                            })
                            .next()
                            .map(|next| {
                                next.dyn_ref::<HtmlLiElement>()
                                    .map(|e| {
                                        e.scroll_into_view_with_bool(true);
                                        e.focus().ok()
                                    })
                            });
                    } else {
                        std::iter::successors(
                                active_element.and_then(|e| e.next_element_sibling()),
                                |x| { x.next_element_sibling() },
                            )
                            .filter(|x| {
                                x.dyn_ref::<HtmlLiElement>()
                                    .map(|e| e.class_list().contains("active"))
                                    .unwrap_or(false)
                            })
                            .next()
                            .map(|next| {
                                next.dyn_ref::<HtmlLiElement>()
                                    .map(|e| {
                                        e.scroll_into_view_with_bool(true);
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
                        .filter(|x| {
                            x.dyn_ref::<HtmlLiElement>()
                                .map(|e| e.class_list().contains("active"))
                                .unwrap_or(false)
                        })
                        .next()
                        .map(|next| {
                            next.dyn_ref::<HtmlLiElement>()
                                .map(|e| {
                                    e.scroll_into_view_with_bool(true);
                                    e.focus().ok()
                                })
                        });
                }
            }
        >
            <input
                id=name
                class="rounded padded"
                node_ref=input_ref
                bind:value=(input_value, set_input_value)
                autocomplete="off"
                on:keydown=move |e| {
                    set_active(true);
                    if e.key_code() == 13 {
                        e.prevent_default();
                        e.stop_propagation();
                        set_active(false);
                        on_select(value().ok_or(input_value()));
                            set_input_value(String::new());
                    }
                }
            />
            <ul
                class="background-3 vertical flex-1 rounded"
                style:gap="1px"
                style:position="absolute"
                style:top="100%"
                style:width="100%"
                style:overflow="hidden"
                style:display=move || if active() { "flex" } else { "none" }
                role="listbox"
                class:active=active
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
                                class="interactive"
                                class:active=move || filter(&input_value(), &item())
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
                                        on_select(Ok(item()));
                                        set_value(Some(item()));
                                        set_input_value(String::new());
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
                                        on_select(Ok(item()));
                                        set_value(Some(item()));
                                        set_active(false);
                                        set_input_value(String::new());
                                    }
                                }
                            >
                                {move || item_view(item())}
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
