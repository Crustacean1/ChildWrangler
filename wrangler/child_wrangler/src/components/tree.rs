use std::collections::HashSet;
use std::iter;
use std::sync::Arc;

use leptos::either::Either;
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_navigate, use_params};
use uuid::Uuid;

use crate::components::loader::Loader;
use crate::icons::arrow_down::ArrowDown;
use crate::pages::attendance_page::{AttendanceParams, GroupVersion};
use crate::services::group::get_groups;
use crate::services::student::get_students;
use dto::group::GroupDto;
use dto::student::StudentDto;

#[derive(Clone, Debug)]
pub struct TreeItem {
    pub id: Uuid,
    pub name: String,
    pub is_student: bool,
    pub parent: Option<Uuid>,
}

#[component]
pub fn InnerTree() -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context::<GroupVersion>().unwrap();

    let groups = Resource::new(
        move || (group_version()),
        |i| async move { get_groups().await },
    );
    let students = Resource::new(
        move || (group_version()),
        |i| async move { get_students().await },
    );

    let (expanded, set_expanded) = signal(HashSet::new());

    let params = use_params::<AttendanceParams>();
    let params = move || params.read();

    let target = move || {
        params()
            .as_ref()
            .ok()
            .and_then(|attendance| attendance.target)
    };

    view! {
        <Loader>
            {move || Suspend::new(async move {
                let groups = groups.await?;
                let students = students.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(view! { <Test groups students target expanded set_expanded /> })
            })}
        </Loader>
    }
}

#[component]
fn Test(
    groups: Vec<GroupDto>,
    students: Vec<StudentDto>,
    expanded: ReadSignal<HashSet<Uuid>>,
    set_expanded: WriteSignal<HashSet<Uuid>>,
    target: impl Fn() -> Option<Uuid> + Send + Sync + Copy + 'static,
) -> impl IntoView {
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

    let entities = Arc::new({
        let mut entities = groups.chain(students).collect::<Vec<_>>();
        entities.sort_by_key(|e| e.name.clone());
        entities
    });

    let all_expanded = {
        let entities = entities.clone();
        move || {
            iter::successors(target(), |item| {
                entities
                    .iter()
                    .find(|e| e.id == *item)
                    .and_then(|e| e.parent)
            })
            .chain(expanded().into_iter())
            .collect::<HashSet<_>>()
        }
    };

    view! {
        <div class="overflow-auto scrollbar-hide min-w-xs rounded-xl bg-gray-900 outline outline-white/15 p-2 m-0.5">
            <ul class="flex flex-col">
                {entities
                    .iter()
                    .filter(|item| item.parent.is_none())
                    .map(|item| {
                        view! {
                            <TreeNode
                                root=item.clone()
                                groups=entities.clone()
                                expanded
                                set_expanded
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[component]
fn TreeNode(
    root: TreeItem,
    groups: Arc<Vec<TreeItem>>,
    expanded: ReadSignal<HashSet<Uuid>>,
    set_expanded: WriteSignal<HashSet<Uuid>>,
) -> impl IntoView {
    let id = root.id;
    let is_student = root.is_student;
    let name = root.name.clone();

    let dropzone_ref: NodeRef<html::Li> = NodeRef::new();
    let navigate = use_navigate();

    let on_toggle_expand = move |_| {
        if expanded().contains(&root.id) {
            set_expanded.write().remove(&root.id);
        } else {
            set_expanded.write().insert(root.id);
        }
    };

    view! {
        <li
            class="flex-row overflow-hidden"
            node_ref=dropzone_ref
            class:expanded=move || !expanded().contains(&root.id)
        >
            <span class="flex-1 flex overflow-hidden rounded-lg">
                <span
                    class="flex-1 btn"
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
                                <button
                                    class="btn "
                                    class:rotate-180=move || expanded().contains(&id)
                                    on:click=on_toggle_expand
                                >
                                    <ArrowDown />
                                </button>
                            },
                        )
                    }
                }}
            </span>
            {move || {
                if expanded().contains(&id) {
                    Either::Left(

                        view! {
                            <ul class="flex flex-col " style:padding-left="1em">
                                {groups
                                    .iter()
                                    .filter(|g| g.parent == Some(root.id))
                                    .map(|g| {
                                        view! {
                                            <TreeNode
                                                groups=groups.clone()
                                                root=g.clone()
                                                expanded
                                                set_expanded
                                            />
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </ul>
                        },
                    )
                } else {
                    Either::Right(view! {})
                }
            }}
        </li>
    }
    .into_any()
}
