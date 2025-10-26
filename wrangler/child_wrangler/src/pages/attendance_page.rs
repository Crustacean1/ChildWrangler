use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::params::Params;
use uuid::Uuid;

use crate::components::{modal::Modal, modals::add_catering::AddCateringModal, tree::InnerTree};

#[derive(Params, PartialEq)]
pub struct AttendanceParams {
    pub target: Uuid,
    pub year: u32,
    pub month: u32,
}

#[derive(Clone, Debug)]
pub struct GroupVersion(pub ReadSignal<i32>, pub WriteSignal<i32>);

#[derive(Clone, Debug)]
pub struct AttendanceVersion(pub ReadSignal<i32>, pub WriteSignal<i32>);

#[component]
pub fn AttendancePage() -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();

    let (catering_modal, set_catering_modal) = signal(false);

    view! {
        <div class="flex flex-row flex-1 gap-2">
            <div class="flex flex-col gap-2">
                <InnerTree />
                <button class="btn bg-gray-900" on:click=move |_| set_catering_modal(true)>
                    Dodaj catering
                </button>
            </div>

            <div class="flex flex-col flex-1 gap-2">
                <Outlet />
            </div>
        </div>
        <Modal is_open=catering_modal on_close=move || set_catering_modal(false)>
            <AddCateringModal
                is_open=catering_modal
                on_close=move |created| {
                    if let Some(id) = created {
                        *set_group_version.write() += 1;
                    }
                    set_catering_modal(false)
                }
            />
        </Modal>
    }
}
