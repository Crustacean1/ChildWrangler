use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::HtmlDivElement;
use std::fmt::Display;

use chrono::Duration;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum MsgType {
    Success,
    Warning,
    Error,
}

#[derive(Clone)]
pub struct SnackbarMsg {
    id: Uuid,
    msg_type: MsgType,
    content: String,
}

impl SnackbarMsg {
    pub fn success(content: String) -> Self {
        SnackbarMsg {
            id: Uuid::new_v4(),
            content,
            msg_type: MsgType::Success,
        }
    }

    pub fn error(content: String) -> Self {
        SnackbarMsg {
            id: Uuid::new_v4(),
            content,
            msg_type: MsgType::Error,
        }
    }

    pub fn warning(content: String) -> Self {
        SnackbarMsg {
            id: Uuid::new_v4(),
            content,
            msg_type: MsgType::Warning,
        }
    }
}

pub trait SnackbarContext {
    fn success(&self, msg: &str);
    fn error(&self, msg: &str, e: impl Display);
    fn warning(&self, msg: &str);
}

#[component]
pub fn Snackbar(children: ChildrenFn) -> impl IntoView {
    let (messages, set_messages) = signal(vec![]);

    provide_context(set_messages);

    let div_ref = NodeRef::new();

    Effect::new(move || {
        let div: HtmlDivElement = div_ref.get().expect("sdfa");
        div.style(format!(
            "left: 0; transition-duration: 0.25s; transform: translate(-50%,100%);bottom: {}em",
            messages().len() as f32 * 3.5
        ));
    });

    view! {
        {children()}
        <div class="snackbar-root absolute z-2 w-0" style:bottom="0" style:left="50%" data-testid="snackbar-root">
            <div node_ref=div_ref class="relative gap reverse-vertical align-center">
                <For
                    each=move || {
                        let mut messages = messages();
                        messages.reverse();
                        messages
                    }
                    key=|m: &SnackbarMsg| m.id
                    let:child
                >
                    <div
                        class="snackbar-msg rounded-md p-2 outline-2 select-none w-fit"
                        class:bg-red-600=child.msg_type == MsgType::Error
                        class:bg-green-600=child.msg_type == MsgType::Success
                        on:click=move |_| set_messages.write().retain(|msg| msg.id != child.id)
                    >
                        {child.content}
                    </div>
                </For>
            </div>
        </div>
    }
}

fn insert_message(snck: &Option<WriteSignal<Vec<SnackbarMsg>>>, msg: SnackbarMsg) {
    let id = msg.id;
    snck.map(|ctx| {
        ctx.write().push(msg);
        set_timeout(
            move || {
                ctx.write().retain(|snck| snck.id != id);
            },
            Duration::seconds(5).to_std().unwrap(),
        );
    });
}

impl SnackbarContext for Option<WriteSignal<Vec<SnackbarMsg>>> {
    fn success(&self, msg: &str) {
        let msg = SnackbarMsg::success(msg.to_string());
        insert_message(self, msg);
    }

    fn error(&self, msg: &str, e: impl Display) {
        let msg = SnackbarMsg::error(format!("{} {}", msg, e));
        insert_message(self, msg);
    }

    fn warning(&self, msg: &str) {
        let msg = SnackbarMsg::warning(msg.to_string());
        insert_message(self, msg);
    }
}

pub fn use_snackbar() -> Option<WriteSignal<Vec<SnackbarMsg>>> {
    use_context::<WriteSignal<Vec<SnackbarMsg>>>()
}
