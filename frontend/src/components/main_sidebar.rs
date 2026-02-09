use crate::contexts::auth::AuthContext;
use crate::i18n::t;
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
    let is_admin = auth.has_role("Admin");

    let is_active = |route: &Route| -> &str {
        if std::mem::discriminant(&props.active_route) == std::mem::discriminant(route) {
            "active"
        } else {
            ""
        }
    };

    if !is_authenticated {
        return html! {};
    }

    let nav_links = html! {
        <div class="d-flex flex-column p-3">
            <ul class="nav nav-pills flex-column mb-auto">
                <li class="nav-item">
                    <Link<Route> to={Route::Home} classes={classes!("nav-link", is_active(&Route::Home))}>
                        <i class="bi bi-house-door me-2"></i>{t("nav-dashboard")}
                    </Link<Route>>
                </li>
                <li class="nav-item">
                    <Link<Route> to={Route::Buildings} classes={classes!("nav-link", is_active(&Route::Buildings))}>
                        <i class="bi bi-building me-2"></i>{t("nav-buildings")}
                    </Link<Route>>
                </li>
                <li class="nav-item">
                    <Link<Route> to={Route::Maintenance} classes={classes!("nav-link", is_active(&Route::Maintenance))}>
                        <i class="bi bi-tools me-2"></i>{t("nav-maintenance")}
                    </Link<Route>>
                </li>
                <li class="nav-item">
                    <Link<Route> to={Route::Voting} classes={classes!("nav-link", is_active(&Route::Voting))}>
                        <i class="bi bi-check2-square me-2"></i>{t("nav-voting")}
                    </Link<Route>>
                </li>
                <li class="nav-item">
                    <Link<Route> to={Route::MyProperties} classes={classes!("nav-link", is_active(&Route::MyProperties))}>
                        <i class="bi bi-house me-2"></i>{t("nav-my-properties")}
                    </Link<Route>>
                </li>
            </ul>

            if is_admin_or_manager {
                <hr class="my-3" />
                <h6 class="sidebar-heading px-3 mt-3 mb-1 text-muted text-uppercase">
                    <span>{t("nav-management")}</span>
                </h6>
                <ul class="nav nav-pills flex-column">
                    if is_admin {
                        <li class="nav-item">
                            <Link<Route> to={Route::Admin} classes={classes!("nav-link", is_active(&Route::Admin))}>
                                <i class="bi bi-people me-2"></i>{t("nav-users")}
                            </Link<Route>>
                        </li>
                    }
                    <li class="nav-item">
                        <Link<Route> to={Route::AdminProperties} classes={classes!("nav-link", is_active(&Route::AdminProperties))}>
                            <i class="bi bi-grid-3x3 me-2"></i>{t("nav-properties")}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::AdminAnnouncements} classes={classes!("nav-link", is_active(&Route::AdminAnnouncements))}>
                            <i class="bi bi-megaphone me-2"></i>{t("nav-announcements")}
                        </Link<Route>>
                    </li>
                    <li class="nav-item">
                        <Link<Route> to={Route::MeterManagement} classes={classes!("nav-link", is_active(&Route::MeterManagement))}>
                            <i class="bi bi-speedometer2 me-2"></i>{t("nav-admin-meters")}
                        </Link<Route>>
                    </li>
                </ul>
            }
        </div>
    };

    html! {
        <>
            <nav class="sidebar-desktop bg-light border-end"
                 style="width: 250px; min-height: 100vh; position: fixed; top: 56px; left: 0; overflow-y: auto; padding-bottom: 20px;">
                {nav_links.clone()}
            </nav>

            <div class="offcanvas offcanvas-start sidebar-mobile-toggle" tabindex="-1" id="mobileSidebar"
                 style="top: 56px; width: 250px;">
                <div class="offcanvas-header pb-0">
                    <h6 class="offcanvas-title text-muted text-uppercase small">{t("nav-navigation")}</h6>
                    <button type="button" class="btn-close" data-bs-dismiss="offcanvas"></button>
                </div>
                <div class="offcanvas-body p-0">
                    {nav_links}
                </div>
            </div>
        </>
    }
}
