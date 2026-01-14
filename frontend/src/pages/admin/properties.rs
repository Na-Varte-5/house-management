// Properties administration page: buildings, apartments, and owner assignments.
use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{spinner::Spinner, AdminLayout, ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize, Clone, PartialEq)]
struct Building { id: u64, address: String, construction_year: Option<i32> }

#[derive(Deserialize, Clone, PartialEq)]
struct Apartment { id: u64, building_id: u64, number: String, size_sq_m: Option<f64> }

#[derive(Deserialize, Clone, PartialEq)]
struct UserInfo { id: u64, name: String, email: String }

#[derive(Serialize)]
struct NewBuilding { address: String, construction_year: Option<i32> }

#[derive(Serialize)]
struct NewApartment { building_id: u64, number: String, size_sq_m: Option<f64> }

#[derive(Serialize)]
struct AssignOwnerRequest { user_id: u64 }

/// Admin/manager page focused only on buildings, apartments and owner assignments.
#[function_component(AdminPropertiesPage)]
pub fn admin_properties_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

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
    let all_users = use_state(|| Vec::<UserInfo>::new());
    let user_query = use_state(String::default);

    let deleted_buildings = use_state(|| Vec::<Building>::new());
    let deleted_apartments = use_state(|| Vec::<Apartment>::new());
    let show_deleted = use_state(|| false);

    let pending_delete_building = use_state(|| None::<u64>);
    let pending_delete_apartment = use_state(|| None::<u64>);

    let loading_buildings = use_state(|| true);
    let loading_apartments = use_state(|| false);
    let loading_owners = use_state(|| false);
    let loading_deleted = use_state(|| false);

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
                    match client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid)).await {
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
                    match client.get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid)).await {
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

    // Load deleted when toggle on
    {
        let show = show_deleted.clone();
        let del_b = deleted_buildings.clone();
        let del_a = deleted_apartments.clone();
        let loading = loading_deleted.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(show_deleted.clone(), move |_| {
            if *show {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(list) = client.get::<Vec<Building>>("/buildings/deleted").await {
                        del_b.set(list);
                    }
                    if let Ok(list) = client.get::<Vec<Apartment>>("/apartments/deleted").await {
                        del_a.set(list);
                    }
                    loading.set(false);
                });
            } else {
                del_b.set(Vec::new());
                del_a.set(Vec::new());
                loading.set(false);
            }
            || ()
        });
    }

    // Add building handler
    let on_add_building = {
        let address = address.clone();
        let year = year.clone();
        let buildings = buildings.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

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
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_building = NewBuilding {
                    address: addr,
                    construction_year: year.parse::<i32>().ok(),
                };

                match client.post::<_, serde_json::Value>("/buildings", &new_building).await {
                    Ok(_) => {
                        // Reload buildings
                        if let Ok(list) = client.get::<Vec<Building>>("/buildings").await {
                            buildings.set(list);
                            address.set(String::new());
                            year.set(String::new());
                            success.set(Some("Building added successfully".to_string()));
                        }
                    }
                    Err(e) => error.set(Some(format!("Failed to add building: {}", e))),
                }
            });
        })
    };

    // Add apartment handler
    let on_add_apartment = {
        let apt_number = apt_number.clone();
        let apt_size = apt_size.clone();
        let selected_building = selected_building.clone();
        let apartments = apartments.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

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
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let new_apt = NewApartment {
                        building_id: bid,
                        number: num,
                        size_sq_m: apt_size.parse::<f64>().ok(),
                    };

                    match client.post::<_, serde_json::Value>("/apartments", &new_apt).await {
                        Ok(_) => {
                            // Reload apartments
                            if let Ok(list) = client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid)).await {
                                apartments.set(list);
                                apt_number.set(String::new());
                                apt_size.set(String::new());
                                success.set(Some("Apartment added successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to add apartment: {}", e))),
                    }
                });
            }
        })
    };

    // Delete building
    let on_delete_building = {
        let buildings = buildings.clone();
        let selected_building = selected_building.clone();
        let show_deleted = show_deleted.clone();
        let deleted_buildings = deleted_buildings.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let buildings = buildings.clone();
            let selected_building = selected_building.clone();
            let show_deleted = show_deleted.clone();
            let deleted_buildings = deleted_buildings.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.delete_no_response(&format!("/buildings/{}", id)).await {
                    Ok(_) => {
                        // Reload active buildings
                        if let Ok(list) = client.get::<Vec<Building>>("/buildings").await {
                            buildings.set(list);
                        }
                        // Reload deleted if visible
                        if *show_deleted {
                            if let Ok(list) = client.get::<Vec<Building>>("/buildings/deleted").await {
                                deleted_buildings.set(list);
                            }
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

    // Delete apartment
    let on_delete_apartment = {
        let apartments = apartments.clone();
        let selected_building = selected_building.clone();
        let show_deleted = show_deleted.clone();
        let deleted_apartments = deleted_apartments.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let apartments = apartments.clone();
            let selected_building = selected_building.clone();
            let show_deleted = show_deleted.clone();
            let deleted_apartments = deleted_apartments.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.delete_no_response(&format!("/apartments/{}", id)).await {
                    Ok(_) => {
                        // Reload active apartments
                        if let Some(bid) = *selected_building {
                            if let Ok(list) = client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid)).await {
                                apartments.set(list);
                            }
                        }
                        // Reload deleted if visible
                        if *show_deleted {
                            if let Ok(list) = client.get::<Vec<Apartment>>("/apartments/deleted").await {
                                deleted_apartments.set(list);
                            }
                        }
                        success.set(Some("Apartment deleted".to_string()));
                    }
                    Err(e) => error.set(Some(format!("Failed to delete apartment: {}", e))),
                }
            });
        })
    };

    // Restore building
    let restore_building = {
        let buildings = buildings.clone();
        let deleted_buildings = deleted_buildings.clone();
        let show_deleted = show_deleted.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let buildings = buildings.clone();
            let deleted_buildings = deleted_buildings.clone();
            let show_deleted = show_deleted.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post_empty::<serde_json::Value>(&format!("/buildings/{}/restore", id)).await {
                    Ok(_) => {
                        // Reload active
                        if let Ok(list) = client.get::<Vec<Building>>("/buildings").await {
                            buildings.set(list);
                        }
                        // Reload deleted
                        if *show_deleted {
                            if let Ok(list) = client.get::<Vec<Building>>("/buildings/deleted").await {
                                deleted_buildings.set(list);
                            }
                        }
                        success.set(Some("Building restored".to_string()));
                    }
                    Err(e) => error.set(Some(format!("Failed to restore building: {}", e))),
                }
            });
        })
    };

    // Restore apartment
    let restore_apartment = {
        let apartments = apartments.clone();
        let deleted_apartments = deleted_apartments.clone();
        let selected_building = selected_building.clone();
        let show_deleted = show_deleted.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            let apartments = apartments.clone();
            let deleted_apartments = deleted_apartments.clone();
            let selected_building = selected_building.clone();
            let show_deleted = show_deleted.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post_empty::<serde_json::Value>(&format!("/apartments/{}/restore", id)).await {
                    Ok(_) => {
                        // Reload active apartments
                        if let Some(bid) = *selected_building {
                            if let Ok(list) = client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", bid)).await {
                                apartments.set(list);
                            }
                        }
                        // Reload deleted
                        if *show_deleted {
                            if let Ok(list) = client.get::<Vec<Apartment>>("/apartments/deleted").await {
                                deleted_apartments.set(list);
                            }
                        }
                        success.set(Some("Apartment restored".to_string()));
                    }
                    Err(e) => error.set(Some(format!("Failed to restore apartment: {}", e))),
                }
            });
        })
    };

    // Add owner
    let add_owner = {
        let selected_apartment = selected_apartment.clone();
        let owners = apartment_owners.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |user_id: u64| {
            if let Some(aid) = *selected_apartment {
                // Check if already owner
                if owners.iter().any(|o| o.id == user_id) {
                    return;
                }

                let owners = owners.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let req = AssignOwnerRequest { user_id };

                    match client.post::<_, serde_json::Value>(&format!("/apartments/{}/owners", aid), &req).await {
                        Ok(_) => {
                            // Reload owners
                            if let Ok(list) = client.get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid)).await {
                                owners.set(list);
                                success.set(Some("Owner added successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to add owner: {}", e))),
                    }
                });
            } else {
                error.set(Some("Select an apartment first".into()));
            }
        })
    };

    // Remove owner
    let remove_owner = {
        let selected_apartment = selected_apartment.clone();
        let owners = apartment_owners.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |owner_id: u64| {
            if let Some(aid) = *selected_apartment {
                let owners = owners.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();

                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client.delete_no_response(&format!("/apartments/{}/owners/{}", aid, owner_id)).await {
                        Ok(_) => {
                            // Reload owners
                            if let Ok(list) = client.get::<Vec<UserInfo>>(&format!("/apartments/{}/owners", aid)).await {
                                owners.set(list);
                                success.set(Some("Owner removed successfully".to_string()));
                            }
                        }
                        Err(e) => error.set(Some(format!("Failed to remove owner: {}", e))),
                    }
                });
            }
        })
    };

    // Delete confirmation handlers
    let confirm_delete_building = {
        let pending = pending_delete_building.clone();
        let del = on_delete_building.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(id) = *pending {
                del.emit(id);
            }
            pending.set(None);
        })
    };

    let confirm_delete_apartment = {
        let pending = pending_delete_apartment.clone();
        let del = on_delete_apartment.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(id) = *pending {
                del.emit(id);
            }
            pending.set(None);
        })
    };

    let cancel_delete = {
        let p1 = pending_delete_building.clone();
        let p2 = pending_delete_apartment.clone();
        Callback::from(move |_: MouseEvent| {
            p1.set(None);
            p2.set(None);
        })
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    // Filter users by search query
    let filtered_users: Vec<UserInfo> = {
        let q = user_query.to_lowercase();
        if q.is_empty() {
            (*all_users).clone()
        } else {
            all_users.iter()
                .filter(|u| u.name.to_lowercase().contains(&q) || u.email.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    };

    // Delete confirmation modal
    let show_modal = pending_delete_building.is_some() || pending_delete_apartment.is_some();
    let modal_msg = if pending_delete_building.is_some() {
        "Delete this building and all its apartments?"
    } else if pending_delete_apartment.is_some() {
        "Delete this apartment?"
    } else {
        ""
    };

    let modal = if show_modal {
        html! {
            <div class="modal fade show" style="display:block; background:rgba(0,0,0,.5);" role="dialog">
                <div class="modal-dialog modal-sm">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h6 class="modal-title">{"Confirm Deletion"}</h6>
                            <button type="button" class="btn-close" onclick={cancel_delete.clone()}></button>
                        </div>
                        <div class="modal-body">
                            <p>{modal_msg}</p>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-secondary btn-sm" onclick={cancel_delete.clone()}>{"Cancel"}</button>
                            {
                                if pending_delete_building.is_some() {
                                    html! { <button class="btn btn-danger btn-sm" onclick={confirm_delete_building}>{"Delete Building"}</button> }
                                } else {
                                    html! { <button class="btn btn-danger btn-sm" onclick={confirm_delete_apartment}>{"Delete Apartment"}</button> }
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <AdminLayout title={"Properties".to_string()} active_route={crate::routes::Route::AdminProperties}>
            <div class="container-fluid px-0">
                {modal}

                if let Some(err) = (*error).clone() {
                    <ErrorAlert message={err} on_close={clear_error.clone()} />
                }

                if let Some(msg) = (*success).clone() {
                    <SuccessAlert message={msg} on_close={clear_success.clone()} />
                }

                <div class="row mt-3">
                    // Buildings column
                    <div class="col-md-6 mb-3">
                        <div class="card">
                            <div class="card-header d-flex justify-content-between align-items-center">
                                <span>{"Buildings"}</span>
                                <div class="form-check form-switch small">
                                    <input
                                        class="form-check-input"
                                        type="checkbox"
                                        id="showDeleted"
                                        checked={*show_deleted}
                                        onchange={{
                                            let show_deleted = show_deleted.clone();
                                            Callback::from(move |e: Event| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                show_deleted.set(input.checked());
                                            })
                                        }}
                                    />
                                    <label class="form-check-label" for="showDeleted">{"Show deleted"}</label>
                                </div>
                            </div>
                            <div class="card-body">
                                <form class="row g-2" onsubmit={on_add_building}>
                                    <div class="col-6">
                                        <input
                                            class="form-control form-control-sm"
                                            placeholder="Address"
                                            value={(*address).clone()}
                                            oninput={{
                                                let address = address.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    address.set(input.value());
                                                })
                                            }}
                                        />
                                    </div>
                                    <div class="col-3">
                                        <input
                                            class="form-control form-control-sm"
                                            placeholder="Year"
                                            type="number"
                                            value={(*year).clone()}
                                            oninput={{
                                                let year = year.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    year.set(input.value());
                                                })
                                            }}
                                        />
                                    </div>
                                    <div class="col-3 d-grid">
                                        <button class="btn btn-sm btn-primary" type="submit">{"Add"}</button>
                                    </div>
                                </form>
                                <hr class="my-2" />

                                if *loading_buildings {
                                    <Spinner />
                                } else {
                                    <ul class="list-group list-group-sm mt-2">
                                        {
                                            for buildings.iter().map(|b| {
                                                let id = b.id;
                                                let selected_building = selected_building.clone();
                                                let pending = pending_delete_building.clone();
                                                let is_selected = *selected_building == Some(id);
                                                let item_class = if is_selected {
                                                    "list-group-item list-group-item-action active d-flex justify-content-between align-items-center"
                                                } else {
                                                    "list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                                                };
                                                html! {
                                                    <li class={item_class}>
                                                        <span
                                                            onclick={{Callback::from(move |_| selected_building.set(Some(id)))}}
                                                            style="cursor:pointer; flex-grow: 1;"
                                                        >
                                                            {format!("{} ({:?})", b.address, b.construction_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into()))}
                                                        </span>
                                                        <button
                                                            class="btn btn-sm btn-outline-danger"
                                                            onclick={{Callback::from(move |_| pending.set(Some(id)))}}
                                                        >
                                                            {"Delete"}
                                                        </button>
                                                    </li>
                                                }
                                            })
                                        }
                                    </ul>
                                }

                                if *show_deleted {
                                    <>
                                        <hr class="my-2" />
                                        <h6 class="small fw-semibold">{"Deleted Buildings"}</h6>
                                        if *loading_deleted {
                                            <Spinner />
                                        } else {
                                            <ul class="list-group list-group-sm mt-1">
                                                {
                                                    for deleted_buildings.iter().map(|b| {
                                                        let id = b.id;
                                                        let restore = restore_building.clone();
                                                        html! {
                                                            <li class="list-group-item d-flex justify-content-between align-items-center">
                                                                <span>{format!("{} ({:?})", b.address, b.construction_year)}</span>
                                                                <button
                                                                    class="btn btn-sm btn-outline-success"
                                                                    onclick={{Callback::from(move |_| restore.emit(id))}}
                                                                >
                                                                    {"Restore"}
                                                                </button>
                                                            </li>
                                                        }
                                                    })
                                                }
                                            </ul>
                                        }
                                    </>
                                }
                            </div>
                        </div>
                    </div>

                    // Apartments & Owners column
                    <div class="col-md-6 mb-3">
                        <div class="card">
                            <div class="card-header">{"Apartments & Owners"}</div>
                            <div class="card-body">
                                <form class="row g-2" onsubmit={on_add_apartment}>
                                    <div class="col-4">
                                        <input
                                            class="form-control form-control-sm"
                                            placeholder="Number"
                                            value={(*apt_number).clone()}
                                            oninput={{
                                                let apt_number = apt_number.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    apt_number.set(input.value());
                                                })
                                            }}
                                        />
                                    </div>
                                    <div class="col-4">
                                        <input
                                            class="form-control form-control-sm"
                                            placeholder="Size (m²)"
                                            type="number"
                                            step="0.1"
                                            value={(*apt_size).clone()}
                                            oninput={{
                                                let apt_size = apt_size.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    apt_size.set(input.value());
                                                })
                                            }}
                                        />
                                    </div>
                                    <div class="col-4 d-grid">
                                        <button
                                            class="btn btn-sm btn-primary"
                                            type="submit"
                                            disabled={selected_building.is_none()}
                                        >
                                            {"Add"}
                                        </button>
                                    </div>
                                </form>
                                <hr class="my-2" />

                                if *loading_apartments {
                                    <Spinner />
                                } else {
                                    <ul class="list-group list-group-sm mt-2">
                                        {
                                            for apartments.iter().map(|a| {
                                                let id = a.id;
                                                let selected_apartment = selected_apartment.clone();
                                                let pending = pending_delete_apartment.clone();
                                                let nav = navigator.clone();
                                                let is_selected = *selected_apartment == Some(id);
                                                let item_class = if is_selected {
                                                    "list-group-item list-group-item-action active d-flex justify-content-between align-items-center"
                                                } else {
                                                    "list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                                                };
                                                html! {
                                                    <li class={item_class}>
                                                        <span
                                                            onclick={{Callback::from(move |_| selected_apartment.set(Some(id)))}}
                                                            style="cursor:pointer; flex-grow: 1;"
                                                        >
                                                            {format!("{} ({:.1} m²)", a.number, a.size_sq_m.unwrap_or(0.0))}
                                                        </span>
                                                        <div class="btn-group btn-group-sm">
                                                            <button
                                                                class="btn btn-outline-info"
                                                                onclick={{Callback::from(move |_| nav.push(&Route::ApartmentMeters { apartment_id: id }))}}
                                                            >
                                                                <i class="bi bi-speedometer2"></i> {"Meters"}
                                                            </button>
                                                            <button
                                                                class="btn btn-outline-danger"
                                                                onclick={{Callback::from(move |_| pending.set(Some(id)))}}
                                                            >
                                                                {"Delete"}
                                                            </button>
                                                        </div>
                                                    </li>
                                                }
                                            })
                                        }
                                    </ul>
                                }

                                if *show_deleted {
                                    <>
                                        <hr class="my-2" />
                                        <h6 class="small fw-semibold">{"Deleted Apartments"}</h6>
                                        if *loading_deleted {
                                            <Spinner />
                                        } else {
                                            <ul class="list-group list-group-sm mt-1">
                                                {
                                                    for deleted_apartments.iter().map(|a| {
                                                        let id = a.id;
                                                        let restore = restore_apartment.clone();
                                                        html! {
                                                            <li class="list-group-item d-flex justify-content-between align-items-center">
                                                                <span>{format!("{} ({:.1} m²)", a.number, a.size_sq_m.unwrap_or(0.0))}</span>
                                                                <button
                                                                    class="btn btn-sm btn-outline-success"
                                                                    onclick={{Callback::from(move |_| restore.emit(id))}}
                                                                >
                                                                    {"Restore"}
                                                                </button>
                                                            </li>
                                                        }
                                                    })
                                                }
                                            </ul>
                                        }
                                    </>
                                }

                                // Only show owner management when an apartment is selected
                                if selected_apartment.is_some() {
                                    <>
                                        <hr class="my-2" />
                                        <h6 class="small fw-semibold">{"Owners of Selected Apartment"}</h6>
                                        if *loading_owners {
                                            <Spinner />
                                        } else {
                                            <ul class="list-group list-group-sm mt-1">
                                                {
                                                    for apartment_owners.iter().map(|owner| {
                                                        let owner_id = owner.id;
                                                        let remove = remove_owner.clone();
                                                        html! {
                                                            <li class="list-group-item d-flex justify-content-between align-items-center">
                                                                <span>{format!("{} ({})", owner.name, owner.email)}</span>
                                                                <button
                                                                    class="btn btn-sm btn-outline-danger"
                                                                    onclick={{Callback::from(move |_| remove.emit(owner_id))}}
                                                                >
                                                                    {"Remove"}
                                                                </button>
                                                            </li>
                                                        }
                                                    })
                                                }
                                            </ul>
                                        }

                                        <hr class="my-2" />
                                    </>
                                } else {
                                    <>
                                        <hr class="my-2" />
                                        <div class="alert alert-info small mb-0">
                                            <i class="bi bi-info-circle me-2"></i>
                                            {"Select an apartment to manage its owners"}
                                        </div>
                                    </>
                                }
                                // Only show "Assign Owner" search when an apartment is selected
                                if selected_apartment.is_some() {
                                    <>
                                        <h6 class="small fw-semibold">{"Assign Owner"}</h6>
                                        <input
                                            class="form-control form-control-sm mb-2"
                                            placeholder="Search users..."
                                            value={(*user_query).clone()}
                                            oninput={{
                                                let user_query = user_query.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    user_query.set(input.value());
                                                })
                                            }}
                                        />
                                        <div class="list-group list-group-sm" style="max-height: 200px; overflow-y: auto;">
                                            {
                                                for filtered_users.iter().map(|u| {
                                                    let user_id = u.id;
                                                    let add = add_owner.clone();
                                                    html! {
                                                        <button
                                                            type="button"
                                                            class="list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                                                            onclick={{Callback::from(move |_| add.emit(user_id))}}
                                                        >
                                                            <span class="me-2">{&u.name}</span>
                                                            <span class="small text-muted">{&u.email}</span>
                                                        </button>
                                                    }
                                                })
                                            }
                                        </div>
                                    </>
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AdminLayout>
    }
}
