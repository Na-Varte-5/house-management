use yew::prelude::*;
use serde::Deserialize;
use crate::components::spinner::Spinner;
use crate::components::AdminLayout;
use crate::utils::{auth::current_user, api::api_url};

#[derive(Deserialize, Clone, PartialEq)]
struct Building { id: u64, address: String, construction_year: Option<i32> }
#[derive(Deserialize, Clone, PartialEq)]
struct Apartment { id: u64, building_id: u64, number: String, size_sq_m: Option<f64> }

/// Admin/manager page focused only on buildings, apartments and owner assignments.
#[function_component(AdminPropertiesPage)]
pub fn admin_properties_page() -> Html {
    let user = current_user();
    let can_manage = user
        .as_ref()
        .map(|u| u.roles.iter().any(|r| r == "Admin" || r == "Manager"))
        .unwrap_or(false);
    if !can_manage {
        return html! {<div class="container mt-4"><div class="alert alert-danger">{"Access denied"}</div></div>};
    }

    // State copied from ManagePage, but scoped to properties only.
    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| None::<u64>);
    let message = use_state(|| None::<String>);

    let address = use_state(String::default);
    let year = use_state(String::default);
    let apt_number = use_state(String::default);
    let apt_size = use_state(String::default);
    let pending_delete_building = use_state(|| None::<u64>);
    let pending_delete_apartment = use_state(|| None::<u64>);

    let selected_apartment = use_state(|| None::<u64>);
    let apartment_owners = use_state(|| Vec::<(u64,String,String)>::new()); // (id,name,email)
    let all_users = use_state(|| Vec::<(u64,String,String)>::new());
    let user_query = use_state(String::default);

    let deleted_buildings = use_state(|| Vec::<Building>::new());
    let deleted_apartments = use_state(|| Vec::<Apartment>::new());
    let show_deleted = use_state(|| false);

    let loading_buildings = use_state(|| false);
    let loading_apartments = use_state(|| false);
    let loading_owners = use_state(|| false);
    let loading_deleted = use_state(|| false);

    // ...reuse effects and handlers from ManagePage for buildings/apartments/owners...

    html! {
        <AdminLayout title={"Properties".to_string()} active_route={crate::routes::Route::AdminProperties}>
            <div class="container-fluid px-0">
                // ...reuse the buildings and apartments cards from ManagePage, without announcements...
            </div>
        </AdminLayout>
    }
}

// Backward-compat shim that re-exports the admin properties page from the new admin module.
pub use crate::pages::admin::properties::AdminPropertiesPage;
