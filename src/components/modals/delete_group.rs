use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    dtos::group::GroupInfoDto,
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
        <h2 class="h2">Usuń grupę</h2>
        <div style:max-width="20em">
            Czy na pewno chcesz usunąć grupę <em>{format!("{} ", info.name)}</em>oraz
            <em>{format!("{} grup ", info.group_count)}</em>i
            <em>{format!("{} uczniów ", info.student_count)}</em>którzy do niej należą?
        </div>
        <div class="horizontal gap flex-end">
            <button
                class="interactive padded rounded red"
                on:click=move |_| on_close(false)
                disabled=delete_group.pending()
            >
                Anuluj
            </button>
            <button
                class="interactive padded rounded green"
                on:click=move |_| {
                    delete_group.dispatch(());
                }
                disabled=delete_group.pending()
            >
                Usuń
            </button>
        </div>
    }
}
