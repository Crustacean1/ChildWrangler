use dto::guardian::GuardianDetailDto;
use leptos::prelude::*;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::messages::update_guardian,
};

#[component]
pub fn ModifyGuardianModal(
    details: GuardianDetailDto,
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
        let phone = String::from(phone().trim());
        let phone = if phone.is_empty() { None } else { Some(phone) };
        let dto = GuardianDetailDto {
            id: details.id,
            fullname: name(),
            phone,
            students: details.students.clone(),
        };
        update_guardian.dispatch(dto);
    };

    view! {
        <h2 class="text-lg text-center">Edytuj rodzica</h2>
        <div class="flex flex-col">
            <label>Imię i Nazwisko</label>
            <input class="input" autocomplete="off" bind:value=(name, set_name) />
        </div>
        <div class="flex flex-col">
            <label>Nr. telefonu</label>
            <input class="input" autocomplete="off" bind:value=(phone, set_phone) />
        </div>
        <div class="flex flex-row justify-end gap-2">
            <button
                class="btn cancel"
                on:click=move |_| on_close(false)
                disabled=update_guardian.pending()
            >
                Anuluj
            </button>
            <button class="btn save" on:click=on_save disabled=update_guardian.pending()>
                Zapisz
            </button>
        </div>
    }
}
