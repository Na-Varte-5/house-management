use yew::prelude::*;
use yew_router::prelude::*;
use crate::utils::auth::current_user;
use crate::components::AdminLayout;
use crate::routes::Route;

#[function_component(ManagePage)]
pub fn manage_page() -> Html {
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
        <AdminLayout title={"Dashboard".to_string()} active_route={Route::Manage}>
            <div class="container-fluid px-0 mt-2">
                <div class="alert alert-info small">
                    {"Use the sidebar or the quick links below to manage users, announcements, and properties."}
                </div>
                <div class="row g-3 mt-1">
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{"User Management"}</h5>
                                <p class="card-text small text-muted flex-grow-1">{"View users and assign roles."}</p>
                                <Link<Route> to={Route::Admin} classes="btn btn-sm btn-primary mt-2">{"Go to users"}</Link<Route>>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{"Announcements"}</h5>
                                <p class="card-text small text-muted flex-grow-1">{"Create and manage building/community announcements."}</p>
                                <Link<Route> to={Route::AdminAnnouncements} classes="btn btn-sm btn-primary mt-2">{"Go to announcements"}</Link<Route>>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-4">
                        <div class="card h-100">
                            <div class="card-body d-flex flex-column">
                                <h5 class="card-title">{"Properties"}</h5>
                                <p class="card-text small text-muted flex-grow-1">{"Manage buildings, apartments, and owners."}</p>
                                <Link<Route> to={Route::AdminProperties} classes="btn btn-sm btn-primary mt-2">{"Go to properties"}</Link<Route>>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AdminLayout>
    }
}
