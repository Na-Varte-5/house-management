use yew::prelude::*;
use yew_router::prelude::*;

use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::routes::Route;

#[derive(Properties, PartialEq)]
pub struct AdminSidebarProps {
    pub active_route: Route,
}

/// Vertical sidebar navigation for privileged users (Admin / Manager).
#[function_component(AdminSidebar)]
pub fn admin_sidebar(props: &AdminSidebarProps) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let is_admin = auth.has_role("Admin");
    let is_manager_or_admin = auth.is_admin_or_manager();

    if !is_manager_or_admin {
        return html! {};
    }

    let nav_link_classes = |route: &Route| {
        if *route == props.active_route {
            "list-group-item list-group-item-action active"
        } else {
            "list-group-item list-group-item-action"
        }
    };

    html! {
        <div class="card shadow-sm">
            <div class="card-header py-2">
                <span class="fw-semibold">{ t("sidebar-management") }</span>
            </div>
            <div class="list-group list-group-flush small">
                if is_admin {
                    <Link<Route> to={Route::Admin} classes={nav_link_classes(&Route::Admin)}>{ t("sidebar-user-management") }</Link<Route>>
                }
                if is_manager_or_admin {
                    <Link<Route> to={Route::AdminAnnouncements} classes={nav_link_classes(&Route::AdminAnnouncements)}>{ t("sidebar-admin-announcements") }</Link<Route>>
                    <Link<Route> to={Route::AdminProperties} classes={nav_link_classes(&Route::AdminProperties)}>{ t("sidebar-admin-properties") }</Link<Route>>
                    <Link<Route> to={Route::MeterManagement} classes={nav_link_classes(&Route::MeterManagement)}>{ t("sidebar-meters") }</Link<Route>>
                }
            </div>
        </div>
    }
}
