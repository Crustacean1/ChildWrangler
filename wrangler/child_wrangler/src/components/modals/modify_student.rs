use leptos::prelude::*;

use crate::dtos::details::StudentDetailsDto;

#[component]
pub fn ModifyStudentModal(
    on_close: impl Fn(bool) + Send + Sync + Copy + 'static,
    student: StudentDetailsDto,
) -> impl IntoView {

}
