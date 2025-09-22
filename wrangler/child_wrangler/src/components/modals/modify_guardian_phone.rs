use dto::{catering::GuardianDetailDto, messages::GuardianDetails};
use leptos::prelude::*;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::messages::update_guardian,
};

#[component]
pub fn ModifyGuardianModal(
    details: GuardianDetails,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let (name, set_name) = signal(details.fullname);
    let (phone, set_phone) = signal(details.phone.unwrap_or(String::new()));

    let update_guardian = Action::new(move |dto: &GuardianDetailDto| {
        let dto = dto.clone();
        async move {
            match update_guardian(dto).await {
                Ok(_) => {
                    snackbar.success("Zaktualizowano rodzica");
                    on_close(true);
                }
                Err(e) => {
                    snackbar.error("Nie udało sie zaktualizować rodzica", e);
                }
            }
        }
    });

    let on_save = move |_| {
        let dto = GuardianDetailDto {
            id: details.id,
            fullname: name(),
            phone: phone(),
        };
        update_guardian.dispatch(dto);
    };

    view! {
        <h2 class="h2">Edytuj rodzica</h2>
        <div class="vertical">
            <label>Nazwa</label>
            <input class="padded rounded" autocomplete="off" bind:value=(name, set_name) />
        </div>
        <div class="vertical">
            <label>Nr. telefonu</label>
            <input class="padded rounded" autocomplete="off" bind:value=(phone, set_phone) />
        </div>
        <div class="horizontal gap flex-end">
            <button
                class="interactive rounded padded red"
                on:click=move |_| on_close(false)
                disabled=update_guardian.pending()
            >
                Anuluj
            </button>
            <button
                class="interactive rounded padded green"
                on:click=on_save
                disabled=update_guardian.pending()
            >
                Zapisz
            </button>
        </div>
    }
}
