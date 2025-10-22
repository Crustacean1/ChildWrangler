use dto::{
    details::{EntityDto, GroupDetailsDto, StudentDetailsDto},
    group::GroupDto,
};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_params;
use uuid::Uuid;

use crate::{
    components::{
        calendar::Calendar,
        loader::Loader,
        modal::Modal,
        modals::{
            add_group::AddGroupModal, add_student::AddStudentModal, delete_group::DeleteGroupModal,
            delete_student::DeleteStudentModal, modify_group::ModifyGroupModal,
        },
    },
    icons::{add_group::AddGroupIcon, add_user::AddUserIcon, delete::DeleteIcon, edit::EditIcon},
    pages::attendance_page::{AttendanceParams, GroupVersion},
    services::group::{get_breadcrumbs, get_details},
};

#[component]
pub fn DetailPage() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 flex-1">
            <div class="bg-gray-900 outline outline-white/15 rounded-xl p-2 m-0.5">
                <InfoPage />
            </div>
            <Calendar />
        </div>
    }
}

#[component]
pub fn InfoPage() -> impl IntoView {
    let params = use_params::<AttendanceParams>();
    let id = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.target)
            .unwrap_or_default()
    };
    let info = Resource::new(id, |id| async move { get_details(id).await });
    let trail = Resource::new(id, |id| async move { get_breadcrumbs(id).await });
    view! {
        <Loader>
            {move || Suspend::new(async move {
                let info = info.await?;
                let trail = trail.await?;
                Ok::<
                    _,
                    ServerFnError,
                >(
                    match info {
                        EntityDto::Student(student) => {
                            Either::Left(
                                Either::Left(Either::Left(view! { <Student student trail /> })),
                            )
                        }
                        EntityDto::Group(group) => {
                            Either::Left(
                                Either::Left(Either::Right(view! { <NonemptyGroup group trail /> })),
                            )
                        }
                        EntityDto::StudentGroup(group) => {
                            Either::Left(
                                Either::Right(Either::Right(view! { <StudentGroup group trail /> })),
                            )
                        }
                        EntityDto::LeafGroup(group) => {
                            Either::Left(
                                Either::Right(Either::Left(view! { <EmptyGroup group trail /> })),
                            )
                        }
                        EntityDto::Catering(catering) => {
                            Either::Right(view! { <Catering catering trail /> })
                        }
                    },
                )
            })}
        </Loader>
    }
}

#[component]
pub fn Breadcrumb(trail: Vec<GroupDto>) -> impl IntoView {
    view! {
        <nav class="flex flex-1" aria-label="Breadcrumb">
            <ol class="inline-flex items-center space-x-1 md:space-x-2 rtl:space-x-reverse">
                <For each=move || trail.clone() key=|g| g.id let:item>
                    <li>
                        <div class="flex items-center">
                            <svg
                                class="rtl:rotate-180 w-3 h-3 text-gray-400 mx-1"
                                aria-hidden="true"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 6 10"
                            >
                                <path
                                    stroke="currentColor"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="m1 9 4-4-4-4"
                                />
                            </svg>
                            <a
                                class="ms-1 text-sm font-medium text-gray-500 md:ms-2 dark:text-gray-400 md:hover:text-gray-200 align-self-center"
                                href=""
                            >
                                {item.name}
                            </a>
                        </div>
                    </li>
                </For>
            </ol>
        </nav>
    }
}

#[component]
pub fn Catering(catering: GroupDetailsDto, trail: Vec<GroupDto>) -> impl IntoView {
    let (add_group, set_add_group) = signal(false);
    let (edit_group, set_edit_group) = signal(false);
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();

    view! {
        <div class="flex flex-row gap space-between">
            <Breadcrumb trail />

            <div class="flex flex-row gap-1">
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_add_group(true)
                >
                    <AddGroupIcon />
                </button>
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_edit_group(true)
                >
                    <EditIcon />
                </button>
            </div>
        </div>
        <Modal is_open=add_group on_close=move || set_add_group(false)>
            <AddGroupModal
                on_close=move |group| {
                    if let Some(id) = group {
                        *set_group_version.write() += 1;
                    }
                    set_add_group(false);
                }
                parent=catering.id
            />
        </Modal>
        <Modal is_open=edit_group on_close=move || set_edit_group(false)>
            <ModifyGroupModal
                group_name=catering.name.clone()
                on_close=move |group| {
                    if group {
                        *set_group_version.write() += 1;
                    }
                    set_edit_group(false);
                }
                group=catering.id
            />
        </Modal>
    }
}

#[component]
pub fn EmptyGroup(group: GroupDetailsDto, trail: Vec<GroupDto>) -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();

    let (add_group, set_add_group) = signal(false);
    let (add_student, set_add_student) = signal(false);
    let (edit_group, set_edit_group) = signal(false);
    let (delete_group, set_delete_group) = signal(false);

    view! {
        <div class="flex flex-row space-between">
            <Breadcrumb trail />
            <div class="flex flex-row gap-1">
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_add_student(true)
                >
                    <AddUserIcon />
                </button>
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_add_group(true)
                >
                    <AddGroupIcon />
                </button>
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_edit_group(true)
                >
                    <EditIcon />
                </button>
                <button
                    class="md:cursor-pointer md:hover:bg-gray-800 md:active:bg-gray-700 rounded-md p-1"
                    on:click=move |_| set_delete_group(true)
                >
                    <DeleteIcon />
                </button>
            </div>
        </div>
        <Modal is_open=add_group on_close=move || set_add_group(false)>
            <AddGroupModal
                on_close=move |group| {
                    if let Some(id) = group {
                        *set_group_version.write() += 1;
                    }
                    set_add_group(false);
                }
                parent=group.id
            />
        </Modal>
        <Modal is_open=add_student on_close=move || { set_add_student(false) }>
            <AddStudentModal
                on_close=move |student| {
                    if let Some(id) = student {
                        *set_group_version.write() += 1;
                    }
                    set_add_student(false);
                }
                group=group.id
                initial=None
            />
        </Modal>
        <Modal is_open=edit_group on_close=move || { set_edit_group(false) }>
            <ModifyGroupModal
                group_name=group.name.clone()
                on_close=move |edited| {
                    if edited {
                        *set_group_version.write() += 1;
                    }
                    set_edit_group(false);
                }
                group=group.id
            />
        </Modal>
        <Modal is_open=delete_group on_close=move || set_delete_group(false)>
            <DeleteGroupModal
                on_close=move |deleted| {
                    if deleted {
                        *set_group_version.write() += 1;
                    }
                    set_delete_group(false)
                }
                id=group.id
            />
        </Modal>
    }
}

#[component]
pub fn NonemptyGroup(group: GroupDetailsDto, trail: Vec<GroupDto>) -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();
    let (edit_group, set_edit_group) = signal(false);
    let (add_group, set_add_group) = signal(false);
    let (delete_group, set_delete_group) = signal(false);

    view! {
        <div class="flex flex-row space-between">
            <Breadcrumb trail />
            <div class="flex flex-row gap-1">
                <button class="btn" on:click=move |_| { set_add_group(true) }>
                    <AddGroupIcon />
                </button>
                <button class="btn" on:click=move |_| set_edit_group(true)>
                    <EditIcon />
                </button>
                <button class="btn" on:click=move |_| set_delete_group(true)>
                    <DeleteIcon />
                </button>
            </div>
        </div>
        <Modal is_open=edit_group on_close=move || set_edit_group(false)>
            <ModifyGroupModal
                group_name=group.name.clone()
                on_close=move |modified| {
                    if modified {
                        *set_group_version.write() += 1;
                    }
                    set_edit_group(false)
                }
                group=group.id
            />
        </Modal>
        <Modal is_open=add_group on_close=move || set_add_group(false)>
            <AddGroupModal
                on_close=move |group| {
                    if let Some(id) = group {
                        *set_group_version.write() += 1;
                    }
                    set_add_group(false)
                }
                parent=group.id
            />
        </Modal>
        <Modal is_open=delete_group on_close=move || set_delete_group(false)>
            <DeleteGroupModal
                on_close=move |deleted| {
                    if deleted {
                        *set_group_version.write() += 1;
                    }
                    set_delete_group(false)
                }
                id=group.id
            />
        </Modal>
    }
}

#[component]
pub fn StudentGroup(group: GroupDetailsDto, trail: Vec<GroupDto>) -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();
    let (edit_group, set_edit_group) = signal(false);
    let (add_student, set_add_student) = signal(false);
    let (delete_group, set_delete_group) = signal(false);

    view! {
        <div class="flex flex-row space-between">
            <Breadcrumb trail />
            <div class="flex flex-row gap-1">
                <button class="btn" on:click=move |_| { set_add_student(true) }>
                    <AddUserIcon />
                </button>
                <button class="btn" on:click=move |_| set_edit_group(true)>
                    <EditIcon />
                </button>
                <button class="btn" on:click=move |_| set_delete_group(true)>
                    <DeleteIcon />
                </button>
            </div>
        </div>
        <Modal is_open=edit_group on_close=move || set_edit_group(false)>
            <ModifyGroupModal
                group_name=group.name.clone()
                on_close=move |modified| {
                    if modified {
                        *set_group_version.write() += 1;
                    }
                    set_edit_group(false)
                }
                group=group.id
            />
        </Modal>
        <Modal is_open=add_student on_close=move || set_add_student(false)>
            <AddStudentModal
                on_close=move |student| {
                    if let Some(id) = student {
                        *set_group_version.write() += 1;
                    }
                    set_add_student(false)
                }
                group=group.id
                initial=None
            />
        </Modal>
        <Modal is_open=delete_group on_close=move || set_delete_group(false)>
            <DeleteGroupModal
                on_close=move |deleted| {
                    if deleted {
                        *set_group_version.write() += 1;
                    }
                    set_delete_group(false)
                }
                id=group.id
            />
        </Modal>
    }
}

#[component]
pub fn Student(student: StudentDetailsDto, trail: Vec<GroupDto>) -> impl IntoView {
    let GroupVersion(group_version, set_group_version) = use_context().unwrap();
    let (delete_student, set_delete_student) = signal(false);
    let (edit_student, set_edit_student) = signal(false);

    let on_delete = move |deleted| {
        set_delete_student(false);
        if deleted {
            *set_group_version.write() += 1;
        }
    };

    view! {
        <div class="flex flex-row space-between">
            <Breadcrumb trail />
            <div class="flex flex-row gap-1">
                <button class="btn" on:click=move |_| set_edit_student(true)>
                    <EditIcon />
                </button>
                <button class="btn" on:click=move |_| set_delete_student(true)>
                    <DeleteIcon />
                </button>
            </div>
        </div>

        <Modal is_open=delete_student on_close=move || set_delete_student(false)>
            <DeleteStudentModal student_id=student.id on_close=on_delete />
        </Modal>
        <Modal is_open=edit_student on_close=move || set_edit_student(false)>
            <AddStudentModal
                group=Uuid::nil()
                initial=Some(student.clone())
                on_close=move |student| {
                    if let Some(id) = student {
                        *set_group_version.write() += 1;
                    }
                    set_edit_student(false)
                }
            />
        </Modal>
    }
}
