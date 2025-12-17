use yew::prelude::*;
use crate::components::announcements::AnnouncementsManage;
use crate::components::AdminLayout;
use crate::contexts::AuthContext;

/// Admin/manager page for managing announcements (create/edit/list/delete).
#[function_component(AdminAnnouncementsPage)]
pub fn admin_announcements_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    <strong>{"Access denied"}</strong>
                    <p class="mb-0 small">{"You need Admin or Manager permissions to access this page."}</p>
                </div>
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
