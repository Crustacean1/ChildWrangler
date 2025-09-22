use dto::group::CreateGroupDto;
use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::group::create_group,
};

#[component]
pub fn AddGroupModal(
    parent: Uuid,
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let snackbar = use_snackbar();

    let save_group = Action::new(move |dto: &CreateGroupDto| {
        let dto = dto.clone();
        async move {
            match create_group(dto).await {
                Ok(id) => {
                    snackbar.success("Dodano grupę");
                    on_close(Some(id))
                }
                Err(e) => snackbar.error("Nie udało się dodać grupy", e),
            }
        }
    });

    let on_save = move |_| {
        let dto = CreateGroupDto {
            name: name(),
            parent,
        };
        save_group.dispatch(dto);
    };

    view! {
        <h2 class="h2">Dodaj grupę</h2>
        <div class="vertical">
            <label for="name">Nazwa</label>
            <input class="padded rounded" id="name" bind:value=(name, set_name) />
        </div>
        <div class="horizontal gap flex-end">
            <button
                class="interactive rounded padded red"
                on:click=move |_| on_close(None)
                disabled=save_group.pending()
            >
                Anuluj
            </button>
            <button
                class="interactive rounded padded green"
                on:click=on_save
                disabled=save_group.pending()
            >
                Zapisz
            </button>
        </div>
    }
}
