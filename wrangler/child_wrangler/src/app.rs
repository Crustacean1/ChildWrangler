use chrono::{Datelike, Utc};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{A, Outlet, ParentRoute, Redirect, Route, Router, Routes},
    path,
};

use crate::{
    components::{searchbar::Searchbar, snackbar::Snackbar},
    pages::{
        attendance_dashboard::AttendanceDashboard,
        attendance_page::{AttendancePage, AttendanceVersion, GroupVersion},
        detail_page::DetailPage,
        guardian_contact_details::GuardianContactDetails,
        message_dashboard::MessageDashboard,
        message_page::MessagePage,
        unknown_contact_details::UnknownContactDetails,
    },
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (group_version, set_group_version) = signal(0);
    let (attendance_version, set_attendance_version) = signal(0);
    provide_context(GroupVersion(group_version, set_group_version));
    provide_context(AttendanceVersion(
        attendance_version,
        set_attendance_version,
    ));

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/child-wrangler.css" />

        // sets the document title
        <Title text="Child Wrangler" />

        // content for this welcome page
        <Router>
            <Snackbar>
                <Routes fallback=|| "Nie ma takiej strony".into_view()>
                    <ParentRoute
                        path=path!("/")
                        view=|| {
                            view! {
                                <nav class="rounded background-2 padded horizontal">
                                        <Searchbar />
                                    <div class="horizontal flex-1 flex-end gap align-stretch padded">
                                        <A href="/attendance">
                                            <span class="interactive rounded padded" >
                                                Obecność
                                            </span>
                                        </A>
                                        <A href="/messages">
                                            <span class="interactive rounded padded" >
                                                Wiadomości
                                            </span>
                                        </A>
                                    </div>
                                </nav>
                                <main>
                                    <Outlet />
                                </main>
                            }
                        }
                    >
                        <ParentRoute path=path!("messages") view=MessagePage>
                            <Route path=path!("/") view=MessageDashboard />
                            <Route
                                path=path!("/guardian/:id")
                                view=|| {
                                    view! { <GuardianContactDetails /> }
                                }
                            />
                            <Route
                                path=path!("/unknown/:phone")
                                view=|| {
                                    view! { <UnknownContactDetails /> }
                                }
                            />
                        </ParentRoute>
                        <ParentRoute path=path!("attendance") view=AttendancePage>
                            <Route path=path!(":target/:year/:month") view=DetailPage />
                            <Route path=path!("/") view=AttendanceDashboard />
                            <Route
                                path=path!(":target")
                                view=|| {
                                    let current = Utc::now();
                                    view! {
                                        <Redirect path=format!(
                                            "{}/{}",
                                            current.year(),
                                            current.month(),
                                        ) />
                                    }
                                }
                            />

                        </ParentRoute>
                        <Route path=path!("/") view=|| view! { <Redirect path="/attendance" /> } />
                    </ParentRoute>
                </Routes>
            </Snackbar>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
