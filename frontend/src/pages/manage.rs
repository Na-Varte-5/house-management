use crate::components::AdminLayout;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(ManagePage)]
pub fn manage_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    <strong>{t("admin-access-denied")}</strong>
                    <p class="mb-0 small">{t("admin-need-permission")}</p>
                </div>
            </div>
        };
    }

    html! {
        <AdminLayout title={t("manage-dashboard-title")} active_route={Route::Manage}>
            <div class="container-fluid px-0 mt-2">
                <div class="alert alert-info small">
                    {t("manage-dashboard-desc")}
                </div>
                <div class="row g-3 mt-1">
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{t("manage-user-management")}</h5>
                                <p class="card-text small text-muted flex-grow-1">{t("manage-user-management-desc")}</p>
                                <Link<Route> to={Route::Admin} classes="btn btn-sm btn-primary mt-2">{t("manage-go-to-users")}</Link<Route>>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{t("manage-announcements")}</h5>
                                <p class="card-text small text-muted flex-grow-1">{t("manage-announcements-desc")}</p>
                                <Link<Route> to={Route::AdminAnnouncements} classes="btn btn-sm btn-primary mt-2">{t("manage-go-to-announcements")}</Link<Route>>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{t("manage-properties")}</h5>
                                <p class="card-text small text-muted flex-grow-1">{t("manage-properties-desc")}</p>
                                <Link<Route> to={Route::AdminProperties} classes="btn btn-sm btn-primary mt-2">{t("manage-go-to-properties")}</Link<Route>>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AdminLayout>
    }
}
