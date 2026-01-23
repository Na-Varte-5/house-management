use crate::contexts::auth::AuthContext;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MainSidebarProps {
    pub active_route: Route,
}

#[function_component(MainSidebar)]
pub fn main_sidebar(props: &MainSidebarProps) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let is_authenticated = auth.is_authenticated();
    let is_admin_or_manager = auth.is_admin_or_manager();

    // Helper to determine if a route is active
    let is_active = |route: &Route| -> &str {
        if std::mem::discriminant(&props.active_route) == std::mem::discriminant(route) {
            "active"
        } else {
            ""
        }
    };

    // Helper for admin route matching (matches any admin/* route)
    let is_admin_section_active = matches!(
        props.active_route,
        Route::Admin | Route::AdminAnnouncements | Route::AdminProperties | Route::MeterManagement
    );

    if !is_authenticated {
        return html! {};
    }

    html! {
        <nav class="bg-light border-end" style="width: 250px; min-height: 100vh; position: fixed; top: 56px; left: 0; overflow-y: auto; padding-bottom: 20px;">
            <div class="d-flex flex-column p-3">
                // Main navigation section
                <ul class="nav nav-pills flex-column mb-auto">
                    <li class="nav-item">
                        <Link<Route> to={Route::Home} classes={classes!("nav-link", is_active(&Route::Home))}>
                            <i class="bi bi-house-door me-2"></i>
                            {"Dashboard"}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::Buildings} classes={classes!("nav-link", is_active(&Route::Buildings))}>
                            <i class="bi bi-building me-2"></i>
                            {"Buildings"}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::Maintenance} classes={classes!("nav-link", is_active(&Route::Maintenance))}>
                            <i class="bi bi-tools me-2"></i>
                            {"Maintenance"}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::Voting} classes={classes!("nav-link", is_active(&Route::Voting))}>
                            <i class="bi bi-check2-square me-2"></i>
                            {"Voting"}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::MyProperties} classes={classes!("nav-link", is_active(&Route::MyProperties))}>
                            <i class="bi bi-house me-2"></i>
                            {"My Properties"}
                        </Link<Route>>
                    </li>
                </ul>

                // Admin/Manager section
                if is_admin_or_manager {
                    <hr class="my-3" />
                    <h6 class="sidebar-heading px-3 mt-3 mb-1 text-muted text-uppercase">
                        <span>{"Management"}</span>
                    </h6>
                    <ul class="nav nav-pills flex-column">
                        <li class="nav-item">
                            <Link<Route> to={Route::AdminProperties} classes={classes!("nav-link", if is_admin_section_active && matches!(props.active_route, Route::AdminProperties) { "active" } else { "" })}>
                                <i class="bi bi-grid-3x3 me-2"></i>
                                {"Properties"}
                            </Link<Route>>
                        </li>
                        <li class="nav-item">
                            <Link<Route> to={Route::AdminAnnouncements} classes={classes!("nav-link", if is_admin_section_active && matches!(props.active_route, Route::AdminAnnouncements) { "active" } else { "" })}>
                                <i class="bi bi-megaphone me-2"></i>
                                {"Announcements"}
                            </Link<Route>>
                        </li>
                        <li class="nav-item">
                            <Link<Route> to={Route::MeterManagement} classes={classes!("nav-link", if is_admin_section_active && matches!(props.active_route, Route::MeterManagement) { "active" } else { "" })}>
                                <i class="bi bi-speedometer2 me-2"></i>
                                {"Meters"}
                            </Link<Route>>
                        </li>
                    </ul>
                }
            </div>
        </nav>
    }
}
