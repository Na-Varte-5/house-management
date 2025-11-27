use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::admin_sidebar::AdminSidebar;
use crate::routes::Route;
use crate::utils::auth::current_user;

/// Layout for admin/manager pages: renders a sidebar with privileged actions and a main content area.
#[derive(Properties, PartialEq)]
pub struct AdminLayoutProps {
    /// Page title rendered above the content area.
    pub title: String,
    /// Route used to highlight the active sidebar entry.
    pub active_route: Route,
    /// Main page content.
    #[prop_or_default]
    pub children: Children,
}

#[function_component(AdminLayout)]
pub fn admin_layout(props: &AdminLayoutProps) -> Html {
    let user = current_user();
    let is_privileged = user
        .as_ref()
        .map(|u| u.roles.iter().any(|r| r == "Admin" || r == "Manager"))
        .unwrap_or(false);

    if !is_privileged {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">{"Access denied"}</div>
            </div>
        };
    }

    html! {
        <div class="container-fluid mt-3">
            <div class="row">
                // Sidebar column (visible from md and up)
                <div class="col-12 col-md-3 col-lg-2 mb-3 mb-md-0">
                    <div class="d-none d-md-block">
                        <AdminSidebar active_route={props.active_route.clone()} />
                    </div>
                    // Mobile sidebar toggle
                    <div class="d-md-none mb-2">
                        <button
                            class="btn btn-outline-secondary btn-sm"
                            type="button"
                            data-bs-toggle="collapse"
                            data-bs-target="#adminSidebarCollapse"
                            aria-expanded="false"
                            aria-controls="adminSidebarCollapse"
                        >
                            {"Admin menu"}
                        </button>
                        <div class="collapse mt-2" id="adminSidebarCollapse">
                            <AdminSidebar active_route={props.active_route.clone()} />
                        </div>
                    </div>
                </div>

                <div class="col-12 col-md-9 col-lg-10">
                    <h2 class="mb-3">{ props.title.clone() }</h2>
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
