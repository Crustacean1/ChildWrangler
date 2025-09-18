use leptos::prelude::*;
use uuid::Uuid;

use crate::{
    components::snackbar::{use_snackbar, SnackbarContext},
    services::group::delete_group,
};

#[component]
pub fn DeleteStudentModal(
    student_id: Uuid,
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let snackbar = use_snackbar();

    let delete_student = Action::new(move |_: &()| async move {
        match delete_group(student_id).await {
            Ok(_) => {
                snackbar.success("Usunięto ucznia");
                on_close(true);
            }
            Err(e) => {
                snackbar.error("Nie udało się usunąć ucznia", e);
            }
        }
    });

    view! {
        <h2 class="h2">Usunąć ucznia?</h2>
        <div class="horizontal gap flex-end">
            <button
                class="interactive rounded padded red"
                on:click=move |_| on_close(false)
                disabled=delete_student.pending()
            >
                Anuluj
            </button>
            <button
                class="interactive rounded padded green"
                on:click=move |_| {
                    delete_student.dispatch(());
                }
                disabled=delete_student.pending()
            >
                Usuń
            </button>
        </div>
    }
}
