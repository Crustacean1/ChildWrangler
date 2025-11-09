use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{Datelike, Utc};
use leptos::either::Either;
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_navigate, use_params};
use uuid::Uuid;

use crate::components::general_provider::{GroupResource, StudentResource};
use crate::components::loader::Loader;
use crate::icons::arrow_down::ArrowDown;
use crate::pages::attendance_page::AttendanceParams;
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
    let students = expect_context::<StudentResource>().0;
    let groups = expect_context::<GroupResource>().0;

    let (expanded, set_expanded) = signal(HashSet::new());

    let params = use_params::<AttendanceParams>();
    let params = move || params.read();

    let year = move || {
        params()
            .as_ref()
            .ok()
            .map(|p| p.year)
            .unwrap_or(Utc::now().year() as u32)
    };

    let month = move || {
        params()
            .as_ref()
            .ok()
            .map(|p| p.month)
            .unwrap_or(Utc::now().month())
    };

    //let target = move || params().as_ref().ok().map(|p| p.target).unwrap_or_default();

    //Effect::new(move |_| set_expanded.write().insert(target()));

    view! {
        <Loader>
            {move || Suspend::new(async move {
                let groups = groups.await?;
                let students = students.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(view! { <Test groups students expanded set_expanded year month /> })
            })}
        </Loader>
    }
}

#[component]
fn Test(
    groups: HashMap<Uuid, GroupDto>,
    students: HashMap<Uuid, StudentDto>,
    expanded: ReadSignal<HashSet<Uuid>>,
    set_expanded: WriteSignal<HashSet<Uuid>>,
    year: impl Fn() -> u32 + Send + Sync + Clone + Copy + 'static,
    month: impl Fn() -> u32 + Send + Sync + Clone + Copy + 'static,
) -> impl IntoView {
    let students = students.into_iter().map(|(_, s)| TreeItem {
        is_student: true,
        parent: Some(s.group_id),
        id: s.id,
        name: format!("{} {}", s.name, s.surname),
    });
    let groups = groups.into_iter().map(|(_, g)| TreeItem {
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

    view! {
        <div data-testid="group-tree" class="overflow-auto scrollbar-hide md:w-72 ">
            <ul class="flex flex-col">
                {entities
                    .iter()
                    .filter(|item| item.parent.is_none())
                    .map(|item| {
                        view! {
                            <TreeNode
                                year
                                month
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
    year: impl Fn() -> u32 + Send + Sync + Clone + Copy + 'static,
    month: impl Fn() -> u32 + Send + Sync + Clone + Copy + 'static,
) -> impl IntoView {
    let id = root.id;
    let is_student = root.is_student;
    let name = root.name.clone();

    let dropzone_ref: NodeRef<html::Li> = NodeRef::new();

    let on_toggle_expand = move |_| {
        if expanded().contains(&root.id) {
            set_expanded.write().remove(&root.id);
        } else {
            set_expanded.write().insert(root.id);
        }
    };

    view! {
        <li
            class="flex-row overflow-hidden "
            node_ref=dropzone_ref
            class:expanded=move || !expanded().contains(&root.id)
        >
            <span class="flex-1 flex overflow-hidden rounded-md mt-0.5">
                <a
                    data-testid=format!("tree-link-{}", root.id)
                    class="flex-1 bg-gray-800 md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600 p-2"
                    href=move || format!("/attendance/{}/{}/{}", root.id, year(), month())
                >
                    {name.clone()}
                </a>
                {move || {
                    if is_student {
                        Either::Right(view! {})
                    } else {
                        Either::Left(
                            view! {
                                <button
                                    data-testid=format!("tree-expand-button-{}", root.id)
                                    class="bg-gray-800 md:cursor-pointer md:hover:bg-gray-700 md:active:bg-gray-600 p-2"
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
                            <ul class="flex flex-col pl-3 before:content-[''] before:h-full before:min-w-1 before:bg-gray-500 before:absolute before:left-1 before:rounded-full relative">
                                {groups
                                    .iter()
                                    .filter(|g| g.parent == Some(root.id))
                                    .map(|g| {
                                        view! {
                                            <TreeNode
                                                year
                                                month
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
