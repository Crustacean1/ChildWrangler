use std::collections::HashMap;

use dto::{catering::MealDto, group::GroupDto, student::StudentDto};
use leptos::prelude::*;
use leptos_router::components::Outlet;
use uuid::Uuid;

use crate::services::{
    group::get_groups,
    student::{get_meals, get_students},
};

#[derive(Clone, Copy, Debug)]
pub struct GroupResource(pub Resource<Result<HashMap<Uuid, GroupDto>, ServerFnError>>);

#[derive(Clone, Copy, Debug)]
pub struct StudentResource(pub Resource<Result<HashMap<Uuid, StudentDto>, ServerFnError>>);

#[derive(Clone, Copy, Debug)]
pub struct MealResource(pub Resource<Result<HashMap<Uuid, MealDto>, ServerFnError>>);

#[component]
pub fn GeneralProvider() -> impl IntoView {
    let group_resource = Resource::new(
        move || (),
        |_| async move {
            let groups = get_groups().await;
            groups.map(|groups| {
                groups
                    .into_iter()
                    .map(|s| (s.id, s))
                    .collect::<HashMap<_, _>>()
            })
        },
    );

    let student_resource = Resource::new(
        move || (),
        |_| async move {
            get_students().await.map(|students| {
                students
                    .into_iter()
                    .map(|s| (s.id, s))
                    .collect::<HashMap<_, _>>()
            })
        },
    );

    let meal_resource = Resource::new(
        move || (),
        |_| async move {
            get_meals().await.map(|meals| {
                meals
                    .into_iter()
                    .map(|m| (m.id, m))
                    .collect::<HashMap<_, _>>()
            })
        },
    );

    provide_context(GroupResource(group_resource));
    provide_context(StudentResource(student_resource));
    provide_context(MealResource(meal_resource));

    view! { <Outlet /> }
}
