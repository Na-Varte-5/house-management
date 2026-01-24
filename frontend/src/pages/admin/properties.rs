// Refactored Properties administration page using modular components
use crate::components::properties::*;
use crate::components::{AdminLayout, ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct RenterWithUser {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub user: UserInfo,
}

#[function_component(AdminPropertiesPage)]
pub fn admin_properties_page() -> Html {
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

    // State
    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| None::<u64>);
    let selected_apartment = use_state(|| None::<u64>);

    let address = use_state(String::default);
    let year = use_state(String::default);
    let apt_number = use_state(String::default);
    let apt_size = use_state(String::default);

    let apartment_owners = use_state(|| Vec::<UserInfo>::new());
    let apartment_renters = use_state(|| Vec::<RenterInfo>::new());
    let all_users = use_state(|| Vec::<UserInfo>::new());
    let user_query = use_state(String::default);
    let renter_query = use_state(String::default);

    let loading_buildings = use_state(|| true);
    let loading_apartments = use_state(|| false);
    let loading_owners = use_state(|| false);
    let loading_renters = use_state(|| false);
    let submitting = use_state(|| false);

    let management_tab = use_state(|| "owners");

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    // Load buildings on mount
    {
        let buildings = buildings.clone();
        let loading = loading_buildings.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings").await {
                    Ok(list) => buildings.set(list),
                    Err(e) => error.set(Some(format!("Failed to load buildings: {}", e))),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Load apartments when building selected
    {
        let apartments = apartments.clone();
        let selected = selected_building.clone();
        let loading = loading_apartments.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_building.clone(), move |_| {
            if let Some(bid) = *selected {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                        .await
                    {
                        Ok(list) => apartments.set(list),
                        Err(e) => error.set(Some(format!("Failed to load apartments: {}", e))),
                    }
                    loading.set(false);
                });
            } else {
                apartments.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    // Load owners when apartment selected
    {
        let owners = apartment_owners.clone();
        let selected = selected_apartment.clone();
        let loading = loading_owners.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_apartment.clone(), move |_| {
            if let Some(aid) = *selected {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                        .await
                    {
                        Ok(list) => owners.set(list),
                        Err(e) => error.set(Some(format!("Failed to load owners: {}", e))),
                    }
                    loading.set(false);
                });
            } else {
                owners.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    // Load renters when apartment selected
    {
        let renters = apartment_renters.clone();
        let selected = selected_apartment.clone();
        let loading = loading_renters.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(selected_apartment.clone(), move |_| {
            if let Some(aid) = *selected {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .get::<Vec<RenterWithUser>>(&format!("/apartments/{}/renters", aid))
                        .await
                    {
                        Ok(list) => {
                            let renter_infos: Vec<RenterInfo> = list
                                .into_iter()
                                .map(|r| RenterInfo {
                                    id: r.id,
                                    user_id: r.user_id,
                                    apartment_id: r.apartment_id,
                                    start_date: r.start_date,
                                    end_date: r.end_date,
                                    is_active: r.is_active,
                                    user_name: r.user.name,
                                    user_email: r.user.email,
                                })
                                .collect();
                            renters.set(renter_infos);
                        }
                        Err(e) => error.set(Some(format!("Failed to load renters: {}", e))),
                    }
                    loading.set(false);
                });
            } else {
                renters.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    // Load all users for assignment
    {
        let users = all_users.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<UserInfo>>("/users/public").await {
                    users.set(list);
                }
            });
            || ()
        });
    }

    // Handlers
    let on_select_building = {
        let selected = selected_building.clone();
        let selected_apt = selected_apartment.clone();
        Callback::from(move |id: u64| {
            selected.set(Some(id));
            selected_apt.set(None);
        })
    };

    let on_select_apartment = {
        let selected = selected_apartment.clone();
        Callback::from(move |id: u64| selected.set(Some(id)))
    };

    let on_create_building = {
        let address = address.clone();
        let year = year.clone();
        let buildings = buildings.clone();
        let error = error.clone();
        let success = success.clone();
        let submitting = submitting.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let addr = (*address).clone();
            if addr.trim().is_empty() {
                error.set(Some("Address required".into()));
                return;
            }

            let buildings = buildings.clone();
            let address = address.clone();
            let year = year.clone();
            let error = error.clone();
            let success = success.clone();
            let submitting = submitting.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);
            submitting.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_building = NewBuilding {
                    address: addr,
                    construction_year: year.parse::<i32>().ok(),
                };

                match client
                    .post::<_, serde_json::Value>("/buildings", &new_building)
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client.get::<Vec<Building>>("/buildings").await {
                            buildings.set(list);
                            address.set(String::new());
                            year.set(String::new());
                            success.set(Some("Building created successfully".to_string()));
                        }
                    }
                    Err(e) => error.set(Some(format!("Failed to create building: {}", e))),
                }
                submitting.set(false);
            });
        })
    };

    let on_create_apartment = {
        let apt_number = apt_number.clone();
        let apt_size = apt_size.clone();
        let selected_building = selected_building.clone();
        let apartments = apartments.clone();
        let error = error.clone();
        let success = success.clone();
        let submitting = submitting.clone();
        let token = token.clone();

        Callback::from(move |_| {
            if let Some(bid) = *selected_building {
                let num = (*apt_number).clone();
                if num.trim().is_empty() {
                    error.set(Some("Apartment number required".into()));
                    return;
                }

                let apartments = apartments.clone();
                let apt_number = apt_number.clone();
                let apt_size = apt_size.clone();
                let error = error.clone();
                let success = success.clone();
                let submitting = submitting.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);
                submitting.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let new_apt = NewApartment {
                        building_id: bid,
                        number: num,
                        size_sq_m: apt_size.parse::<f64>().ok(),
                    };

                    match client
                        .post::<_, serde_json::Value>("/apartments", &new_apt)
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                                .await
                            {
                                apartments.set(list);
                                apt_number.set(String::new());
                                apt_size.set(String::new());
                                success.set(Some("Apartment created successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to create apartment: {}", e))),
                    }
                    submitting.set(false);
                });
            }
        })
    };

    let on_delete_building = {
        let buildings = buildings.clone();
        let selected_building = selected_building.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let buildings = buildings.clone();
            let selected_building = selected_building.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!("/buildings/{}", id))
                    .await
                {
                    Ok(_) => {
                        if let Ok(list) = client.get::<Vec<Building>>("/buildings").await {
                            buildings.set(list);
                        }
                        if *selected_building == Some(id) {
                            selected_building.set(None);
                        }
                        success.set(Some("Building deleted".to_string()));
                    }
                    Err(e) => error.set(Some(format!("Failed to delete building: {}", e))),
                }
            });
        })
    };

    let on_delete_apartment = {
        let apartments = apartments.clone();
        let selected_building = selected_building.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let apartments = apartments.clone();
            let selected_building = selected_building.clone();
            let selected_apartment = selected_apartment.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!("/apartments/{}", id))
                    .await
                {
                    Ok(_) => {
                        if let Some(bid) = *selected_building {
                            if let Ok(list) = client
                                .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid))
                                .await
                            {
                                apartments.set(list);
                            }
                        }
                        if *selected_apartment == Some(id) {
                            selected_apartment.set(None);
                        }
                        success.set(Some("Apartment deleted".to_string()));
                    }
                    Err(e) => error.set(Some(format!("Failed to delete apartment: {}", e))),
                }
            });
        })
    };

    let on_assign_owner = {
        let apartment_owners = apartment_owners.clone();
        let selected_apartment = selected_apartment.clone();
        let user_query = user_query.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_owners = apartment_owners.clone();
                let user_query = user_query.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let payload = AssignOwnerRequest { user_id };

                    match client
                        .post::<_, serde_json::Value>(
                            &format!("/apartments/{}/owners", aid),
                            &payload,
                        )
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                                .await
                            {
                                apartment_owners.set(list);
                                user_query.set(String::new());
                                success.set(Some("Owner assigned successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to assign owner: {}", e))),
                    }
                });
            }
        })
    };

    let on_remove_owner = {
        let apartment_owners = apartment_owners.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_owners = apartment_owners.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .delete_no_response(&format!("/apartments/{}/owners/{}", aid, user_id))
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid))
                                .await
                            {
                                apartment_owners.set(list);
                                success.set(Some("Owner removed successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to remove owner: {}", e))),
                    }
                });
            }
        })
    };

    let on_assign_renter = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let renter_query = renter_query.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(
            move |(user_id, start_date, end_date, is_active): (
                u64,
                Option<String>,
                Option<String>,
                bool,
            )| {
                if let Some(aid) = *selected_apartment {
                    let apartment_renters = apartment_renters.clone();
                    let renter_query = renter_query.clone();
                    let error = error.clone();
                    let success = success.clone();
                    let token = token.clone();

                    error.set(None);
                    success.set(None);

                    wasm_bindgen_futures::spawn_local(async move {
                        let client = api_client(token.as_deref());
                        let payload = serde_json::json!({
                            "user_id": user_id,
                            "start_date": start_date,
                            "end_date": end_date,
                            "is_active": is_active,
                        });

                        match client
                            .post::<_, serde_json::Value>(
                                &format!("/apartments/{}/renters", aid),
                                &payload,
                            )
                            .await
                        {
                            Ok(_) => {
                                if let Ok(list) = client
                                    .get::<Vec<RenterWithUser>>(&format!(
                                        "/apartments/{}/renters",
                                        aid
                                    ))
                                    .await
                                {
                                    let renter_infos: Vec<RenterInfo> = list
                                        .into_iter()
                                        .map(|r| RenterInfo {
                                            id: r.id,
                                            user_id: r.user_id,
                                            apartment_id: r.apartment_id,
                                            start_date: r.start_date,
                                            end_date: r.end_date,
                                            is_active: r.is_active,
                                            user_name: r.user.name,
                                            user_email: r.user.email,
                                        })
                                        .collect();
                                    apartment_renters.set(renter_infos);
                                    renter_query.set(String::new());
                                    success.set(Some("Renter assigned successfully".to_string()));
                                }
                            }
                            Err(e) => error.set(Some(format!("Failed to assign renter: {}", e))),
                        }
                    });
                }
            },
        )
    };

    let on_remove_renter = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                let apartment_renters = apartment_renters.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client
                        .delete_no_response(&format!("/apartments/{}/renters/{}", aid, user_id))
                        .await
                    {
                        Ok(_) => {
                            if let Ok(list) = client
                                .get::<Vec<RenterWithUser>>(&format!("/apartments/{}/renters", aid))
                                .await
                            {
                                let renter_infos: Vec<RenterInfo> = list
                                    .into_iter()
                                    .map(|r| RenterInfo {
                                        id: r.id,
                                        user_id: r.user_id,
                                        apartment_id: r.apartment_id,
                                        start_date: r.start_date,
                                        end_date: r.end_date,
                                        is_active: r.is_active,
                                        user_name: r.user.name,
                                        user_email: r.user.email,
                                    })
                                    .collect();
                                apartment_renters.set(renter_infos);
                                success.set(Some("Renter removed successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to remove renter: {}", e))),
                    }
                });
            }
        })
    };

    let on_toggle_renter_active = {
        let apartment_renters = apartment_renters.clone();
        let selected_apartment = selected_apartment.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                // Find current renter and toggle is_active
                let current_renter = apartment_renters.iter().find(|r| r.user_id == user_id);
                if let Some(renter) = current_renter {
                    let new_active_status = !renter.is_active;
                    let apartment_renters = apartment_renters.clone();
                    let error = error.clone();
                    let success = success.clone();
                    let token = token.clone();

                    error.set(None);
                    success.set(None);

                    wasm_bindgen_futures::spawn_local(async move {
                        let client = api_client(token.as_deref());
                        let payload = serde_json::json!({
                            "is_active": new_active_status,
                        });

                        match client
                            .put::<_, serde_json::Value>(
                                &format!("/apartments/{}/renters/{}", aid, user_id),
                                &payload,
                            )
                            .await
                        {
                            Ok(_) => {
                                if let Ok(list) = client
                                    .get::<Vec<RenterWithUser>>(&format!(
                                        "/apartments/{}/renters",
                                        aid
                                    ))
                                    .await
                                {
                                    let renter_infos: Vec<RenterInfo> = list
                                        .into_iter()
                                        .map(|r| RenterInfo {
                                            id: r.id,
                                            user_id: r.user_id,
                                            apartment_id: r.apartment_id,
                                            start_date: r.start_date,
                                            end_date: r.end_date,
                                            is_active: r.is_active,
                                            user_name: r.user.name,
                                            user_email: r.user.email,
                                        })
                                        .collect();
                                    apartment_renters.set(renter_infos);
                                    let status = if new_active_status {
                                        "active"
                                    } else {
                                        "inactive"
                                    };
                                    success.set(Some(format!("Renter marked as {}", status)));
                                }
                            }
                            Err(e) => error.set(Some(format!("Failed to update renter: {}", e))),
                        }
                    });
                }
            }
        })
    };

    html! {
        <AdminLayout title="Properties Management" active_route={Route::AdminProperties}>
            <div class="container-fluid">

                // Alert messages
                if let Some(msg) = (*error).clone() {
                    <ErrorAlert message={msg} on_close={Callback::from(move |_| error.set(None))} />
                }
                if let Some(msg) = (*success).clone() {
                    <SuccessAlert message={msg} on_close={Callback::from(move |_| success.set(None))} />
                }

                <div class="row">
                    // Buildings column
                    <div class="col-md-4">
                        <div class="card">
                            <div class="card-header">
                                <h5 class="mb-0">{"Buildings"}</h5>
                            </div>
                            <div class="card-body">
                                <BuildingList
                                    buildings={(*buildings).clone()}
                                    selected_building_id={*selected_building}
                                    on_select={on_select_building}
                                    on_delete={on_delete_building}
                                    loading={*loading_buildings}
                                />
                                <hr class="my-3" />
                                <BuildingForm
                                    address={(*address).clone()}
                                    year={(*year).clone()}
                                    on_address_change={Callback::from(move |v| address.set(v))}
                                    on_year_change={Callback::from(move |v| year.set(v))}
                                    on_submit={on_create_building}
                                    submitting={*submitting}
                                />
                            </div>
                        </div>
                    </div>

                    // Apartments column
                    <div class="col-md-4">
                        <div class="card">
                            <div class="card-header">
                                <h5 class="mb-0">{"Apartments"}</h5>
                            </div>
                            <div class="card-body">
                                <ApartmentList
                                    apartments={(*apartments).clone()}
                                    selected_apartment_id={*selected_apartment}
                                    on_select={on_select_apartment}
                                    on_delete={on_delete_apartment}
                                    loading={*loading_apartments}
                                    show={selected_building.is_some()}
                                />
                                <hr class="my-3" />
                                <ApartmentForm
                                    number={(*apt_number).clone()}
                                    size={(*apt_size).clone()}
                                    on_number_change={Callback::from(move |v| apt_number.set(v))}
                                    on_size_change={Callback::from(move |v| apt_size.set(v))}
                                    on_submit={on_create_apartment}
                                    submitting={*submitting}
                                    show={selected_building.is_some()}
                                />
                            </div>
                        </div>
                    </div>

                    // Owner/Renter Management column
                    <div class="col-md-4">
                        <div class="card">
                            <div class="card-header">
                                <ul class="nav nav-tabs card-header-tabs" role="tablist">
                                    <li class="nav-item" role="presentation">
                                        <button
                                            class={if *management_tab == "owners" { "nav-link active" } else { "nav-link" }}
                                            onclick={{
                                                let tab = management_tab.clone();
                                                Callback::from(move |_| tab.set("owners"))
                                            }}
                                            type="button"
                                        >
                                            {"Owners"}
                                        </button>
                                    </li>
                                    <li class="nav-item" role="presentation">
                                        <button
                                            class={if *management_tab == "renters" { "nav-link active" } else { "nav-link" }}
                                            onclick={{
                                                let tab = management_tab.clone();
                                                Callback::from(move |_| tab.set("renters"))
                                            }}
                                            type="button"
                                        >
                                            {"Renters"}
                                        </button>
                                    </li>
                                    <li class="nav-item" role="presentation">
                                        <button
                                            class={if *management_tab == "history" { "nav-link active" } else { "nav-link" }}
                                            onclick={{
                                                let tab = management_tab.clone();
                                                Callback::from(move |_| tab.set("history"))
                                            }}
                                            type="button"
                                        >
                                            {"History"}
                                        </button>
                                    </li>
                                </ul>
                            </div>
                            <div class="card-body">
                                if *management_tab == "owners" {
                                    <OwnerManagement
                                        owners={(*apartment_owners).clone()}
                                        all_users={(*all_users).clone()}
                                        user_query={(*user_query).clone()}
                                        on_query_change={Callback::from(move |v| user_query.set(v))}
                                        on_assign={on_assign_owner}
                                        on_remove={on_remove_owner}
                                        loading={*loading_owners}
                                        show={selected_apartment.is_some()}
                                    />
                                } else if *management_tab == "renters" {
                                    <RenterManagement
                                        renters={(*apartment_renters).clone()}
                                        all_users={(*all_users).clone()}
                                        user_query={(*renter_query).clone()}
                                        on_query_change={Callback::from(move |v| renter_query.set(v))}
                                        on_assign={on_assign_renter}
                                        on_remove={on_remove_renter}
                                        on_toggle_active={on_toggle_renter_active}
                                        loading={*loading_renters}
                                        show={selected_apartment.is_some()}
                                    />
                                } else {
                                    // History tab
                                    if let Some(apartment_id) = *selected_apartment {
                                        <PropertyHistoryTimeline apartment_id={apartment_id} />
                                    } else {
                                        <div class="alert alert-info">
                                            <i class="bi bi-info-circle me-2"></i>
                                            {"Select an apartment to view its property history"}
                                        </div>
                                    }
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AdminLayout>
    }
}
