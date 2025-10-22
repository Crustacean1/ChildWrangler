use dto::group::GroupInfoDto;
use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::group::{delete_group, get_group_info},
};

#[component]
pub fn DeleteGroupModal(
    id: Uuid,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let info = Resource::new(|| (), move |_| async move { get_group_info(id).await });

    view! {
        <Suspense fallback=|| view! { <div>Loading</div> }>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let info = info.await?;
                    Ok::<_, ServerFnError>(view! { <DeleteGroupModalInner id on_close info /> })
                })}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn DeleteGroupModalInner(
    id: Uuid,
    info: GroupInfoDto,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let delete_group = Action::new(move |_: &()| async move {
        match delete_group(id).await {
            Ok(_) => {
                snackbar.success("Usunięto grupę");
                on_close(true);
            }
            Err(e) => {
                snackbar.error("Nie udało się usunąć grupy", e);
            }
        }
    });

    view! {
        <h2 class="text-center text-lg">Usuń grupę</h2>
        <div class="flex flex-col gap-2">
            <div class="max-w-72">
                Czy na pewno chcesz usunąć grupę
                <em class="bg-gray-600 rounded-md p-0.5">{format!("{} ", info.name)}</em>oraz
                <em class="bg-gray-600 rounded-md p-0.5">
                    {format!("{} grup ", info.group_count)}
                </em>i
                <em class="bg-gray-600 rounded-md p-0.5">
                    {format!("{} uczniów ", info.student_count)}
                </em>którzy do niej należą?
            </div>
            <div class="flex flex-row gap-2 justify-end">
                <button
                    class="btn cancel"
                    on:click=move |_| on_close(false)
                    disabled=delete_group.pending()
                >
                    Anuluj
                </button>
                <button
                    class="btn save"
                    on:click=move |_| {
                        delete_group.dispatch(());
                    }
                    disabled=delete_group.pending()
                >
                    Usuń
                </button>
            </div>
        </div>
    }
}
