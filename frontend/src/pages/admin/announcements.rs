use yew::prelude::*;
use crate::components::announcements::AnnouncementsManage;
use crate::components::AdminLayout;
use crate::utils::auth::current_user;

/// Admin/manager page for managing announcements (create/edit/list/delete).
#[function_component(AdminAnnouncementsPage)]
pub fn admin_announcements_page() -> Html {
    let user = current_user();
    let can_manage = user
        .as_ref()
        .map(|u| u.roles.iter().any(|r| r == "Admin" || r == "Manager"))
        .unwrap_or(false);

    if !can_manage {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">{"Access denied"}</div>
            </div>
        };
    }

    html! {
        <AdminLayout title={"Announcements".to_string()} active_route={crate::routes::Route::AdminAnnouncements}>
            <div class="container-fluid px-0">
                <AnnouncementsManage />
            </div>
        </AdminLayout>
    }
}
