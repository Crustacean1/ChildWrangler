use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::params::Params;
use uuid::Uuid;

use crate::components::{modal::Modal, modals::add_catering::AddCateringModal, tree::InnerTree};

#[derive(Params, PartialEq)]
pub struct AttendanceParams {
    pub target: Option<Uuid>,
    pub year: Option<u32>,
    pub month: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct GroupVersion(pub ReadSignal<i32>, pub WriteSignal<i32>);

#[component]
pub fn AttendancePage() -> impl IntoView {
    let (catering_modal, set_catering_modal) = signal(false);

    let (group_version, set_group_version) = signal(0);
    provide_context(GroupVersion(group_version, set_group_version));

    view! {
        <div class="horizontal flex-1 gap">
            <div class="background-2 rounded padded vertical gap" style:min-width="20em">
                <InnerTree />
                <button
                    class="interactive rounded padded"
                    on:click=move |_| set_catering_modal(true)
                >
                    Dodaj catering
                </button>
            </div>
            <div class="vertical flex-1 gap">
                <Outlet />
            </div>
        </div>
        <Modal is_open=catering_modal on_close=move || set_catering_modal(false)>
            <AddCateringModal is_open=catering_modal on_close=move |_| set_catering_modal(false) />
        </Modal>
    }
}
