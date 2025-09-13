use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Datelike, Utc};
use leptos::either::Either;
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_navigate, use_params};
use uuid::Uuid;

use crate::dtos::group::GroupDto;
use crate::dtos::student::StudentDto;
use crate::icons::arrow_down::ArrowDown;
use crate::pages::attendance_page::AttendanceParams;
use crate::services::group::get_groups;
use crate::services::student::get_students;

#[derive(Clone, Debug)]
pub struct TreeItem {
    pub id: Uuid,
    pub name: String,
    pub is_student: bool,
    pub parent: Option<Uuid>,
}

#[component]
pub fn InnerTree() -> impl IntoView {
    let groups = Resource::new(|| (), |i| async move { get_groups().await });
    let students = Resource::new(|| (), |i| async move { get_students().await });
    view! {
        <Transition>
            <ErrorBoundary fallback=|_| {
                view! { <div>Error</div> }
            }>
                {move || Suspend::new(async move {
                    let groups = groups.await?;
                    let students = students.await?;
                    Ok::<_, ServerFnError>(view! { <Test groups students /> })
                })}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn Test(groups: Vec<GroupDto>, students: Vec<StudentDto>) -> impl IntoView {
    let students = students.into_iter().map(|s| TreeItem {
        is_student: true,
        parent: Some(s.group_id),
        id: s.id,
        name: format!("{} {}", s.name, s.surname),
    });
    let groups = groups.into_iter().map(|g| TreeItem {
        is_student: false,
        parent: g.parent,
        id: g.id,
        name: g.name,
    });

    let entities = Arc::new(groups.chain(students).collect::<Vec<_>>());

    view! {
        <ul class="vertical gap-2">
            {entities
                .iter()
                .filter(|item| item.parent.is_none())
                .map(|item| {
                    view! { <TreeNode root=item.clone() groups=entities.clone() /> }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
}

#[component]
fn TreeNode(root: TreeItem, groups: Arc<Vec<TreeItem>>) -> impl IntoView {
    let on_drag_end = |_| {};
    let on_drag_start = |_| {};

    let id = root.id;
    let is_student = root.is_student;
    let name = root.name.clone();

    let dropzone_ref: NodeRef<html::Li> = NodeRef::new();
    let navigate = use_navigate();

    view! {
        <li class="flex flex-1 vertical gap" node_ref=dropzone_ref>
            <span
                class="rounded flex-1 flex hide-overflow"
                draggable="true"
                on:dragstart=move |e| {
                    e.stop_propagation();
                    if let Some(dt) = e.data_transfer() {
                        dt.set_drop_effect("move");
                    }
                    on_drag_start(id);
                }
                on:dragend=move |_| { on_drag_end(None) }
            >
                <span
                    class="flex-1 interactive padded center"
                    on:click=move |_| {
                        navigate(&format!("/attendance/{}", root.id), Default::default())
                    }
                >
                    {name.clone()}
                </span>
                {move || {
                    if is_student {
                        Either::Right(view! {})
                    } else {
                        Either::Left(
                            view! {
                                <button class="interactive padded center" on:click=|_| {}>
                                    <ArrowDown />
                                </button>
                            },
                        )
                    }
                }}
                <span
                    class="dropzone-marker"
                    on:dragover=|e| {
                        e.prevent_default();
                    }
                    on:drop=move |e| {
                        dropzone_ref
                            .get()
                            .map(|dropzone| {
                                dropzone
                                    .class(
                                        format!("droptarget {}", if true { "expanded" } else { "" }),
                                    )
                            });
                        on_drag_end(Some(id));
                    }
                    on:dragenter=move |e| {
                        dropzone_ref
                            .get()
                            .map(|dropzone| { dropzone.class(format!("droptarget drag")) });
                    }
                    on:dragleave=move |e| {
                        dropzone_ref
                            .get()
                            .map(|dropzone| { dropzone.class(format!("droptarget ")) });
                    }
                    on:dragend=move |e| {
                        dropzone_ref
                            .get()
                            .map(|dropzone| { dropzone.class(format!("droptarget ")) });
                    }
                ></span>
            </span>
            <ul class="vertical" style:padding-left="1em">
                //<li class="dropzone"></li>
                {groups
                    .iter()
                    .filter(|g| g.parent == Some(root.id))
                    .map(|g| {
                        view! { <TreeNode groups=groups.clone() root=g.clone() /> }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </li>
    }
    .into_any()
}
