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
        <div class="flex flex-col gap-2">
            <h2 class="text-center">Usunąć ucznia?</h2>
            <div class="flex flex-row gap-2">
                <button
                    class="btn cancel"
                    on:click=move |_| on_close(false)
                    disabled=delete_student.pending()
                >
                    Anuluj
                </button>
                <button
                    class="btn save"
                    on:click=move |_| {
                        delete_student.dispatch(());
                    }
                    disabled=delete_student.pending()
                >
                    Usuń
                </button>
            </div>
        </div>
    }
}
