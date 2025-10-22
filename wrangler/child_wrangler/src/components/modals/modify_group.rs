use dto::group::ModifyGroupDto;
use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::group::modify_group,
};

#[component]
pub fn ModifyGroupModal(
    group: Uuid,
    group_name: String,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let (name, set_name) = signal(group_name);
    let snackbar = use_snackbar();

    let modify_group = Action::new(move |dto: &ModifyGroupDto| {
        let dto = dto.clone();
        async move {
            match modify_group(dto).await {
                Ok(_) => {
                    snackbar.success("Grupa została zaktualizowana");
                    on_close(true)
                }
                Err(e) => snackbar.error("Nie udało się dodać grupy", e),
            }
        }
    });

    let on_modify = move |_| {
        let dto = ModifyGroupDto {
            name: name(),
            id: group,
        };
        modify_group.dispatch(dto);
    };

    view! {
        <h2 class="text-center text-lg">Edytuj grupę</h2>
            <div class="flex flex-col gap-2">
        <div class="flex flex-col">
            <label for="name">Nazwa</label>
            <input class="input" id="name" bind:value=(name, set_name) />
        </div>
        <div class="flex flex-row gap-2 justify-end">
            <button
                class="btn cancel"
                on:click=move |_| on_close(false)
                disabled=modify_group.pending()
            >
                Anuluj
            </button>
            <button
                class="btn save"
                on:click=on_modify
                disabled=modify_group.pending()
            >
                Zapisz
            </button>
        </div>
        </div>
    }
}
