use dto::group::CreateGroupDto;
use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::{
        general_provider::GroupVersion,
        snackbar::{use_snackbar, SnackbarContext},
    },
    services::group::create_group,
};

#[component]
pub fn AddGroupModal(
    parent: Uuid,
    on_close: impl Fn(Option<Uuid>) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let snackbar = use_snackbar();
    let group_version = expect_context::<GroupVersion>().0;

    let save_group = Action::new(move |dto: &CreateGroupDto| {
        let dto = dto.clone();
        async move {
            match create_group(dto).await {
                Ok(id) => {
                    snackbar.success("Dodano grupę");
                    *group_version.write() += 1;
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
        <h2 class="text-center text-lg">Dodaj grupę</h2>
        <div class="flex flex-col gap-2">
            <div class="flex flex-col">
                <label for="name">Nazwa</label>
                <input
                    class="p-1 rounded-md bg-gray-600 focus:outline-none"
                    id="name"
                    bind:value=(name, set_name)
                />
            </div>
            <div class="flex flex-row gap-2 justify-end">
                <button
                    class="btn cancel"
                    data-testid="add-group-cancel"
                    on:click=move |_| on_close(None)
                    disabled=save_group.pending()
                >
                    Anuluj
                </button>
                <button
                    class="btn save"
                    on:click=on_save
                    disabled=save_group.pending()
                    data-testid="add-group-save"
                >
                    Zapisz
                </button>
            </div>
        </div>
    }
}
