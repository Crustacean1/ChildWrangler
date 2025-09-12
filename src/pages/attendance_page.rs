use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::components::{modal::Modal, modals::add_catering::AddCateringModal};

#[component]
pub fn AttendancePage() -> impl IntoView {
    let (catering_modal, set_catering_modal) = signal(false);

    view! {
        <div class="horizontal flex-1 gap">
            <div class="background-2 rounded padded vertical gap" style:min-width="20em">
                Tree
                <button
                    class="interactive rounded padded"
                    on:click=move |_| set_catering_modal(true)
                >
                    Dodaj catering
                </button>
            </div>
            <div class="vertical flex-1 gap">
                <div class="background-2 rounded padded">Whatever</div>
                <Outlet />
            </div>
        </div>
        <Modal is_open=catering_modal on_close=move || set_catering_modal(false)>
            <AddCateringModal is_open=catering_modal on_close=move || set_catering_modal(false) />
        </Modal>
    }
}
