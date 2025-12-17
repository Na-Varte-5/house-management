use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Building {
    id: u64,
    address: String,
    construction_year: Option<i32>,
}

#[derive(Serialize)]
struct NewBuilding {
    address: String,
    construction_year: Option<i32>,
}

#[function_component(BuildingsPage)]
pub fn buildings_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let buildings = use_state(|| Vec::<Building>::new());
    let address = use_state(String::default);
    let year = use_state(|| String::new());
    let loading = use_state(|| true);
    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Load buildings on mount
    {
        let buildings = buildings.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings").await {
                    Ok(list) => {
                        buildings.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load buildings: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_submit = {
        let address = address.clone();
        let year = year.clone();
        let buildings = buildings.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let address_val = (*address).clone();
            let year_val = (*year).clone();

            if address_val.is_empty() {
                error.set(Some("Address is required".to_string()));
                return;
            }

            let buildings = buildings.clone();
            let address = address.clone();
            let year = year.clone();
            let submitting = submitting.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            submitting.set(true);
            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_building = NewBuilding {
                    address: address_val,
                    construction_year: year_val.parse::<i32>().ok(),
                };

                match client.post::<_, serde_json::Value>("/buildings", &new_building).await {
                    Ok(_) => {
                        // Reload buildings list
                        match client.get::<Vec<Building>>("/buildings").await {
                            Ok(list) => {
                                buildings.set(list);
                                address.set(String::new());
                                year.set(String::new());
                                success.set(Some("Building added successfully".to_string()));
                            }
                            Err(e) => {
                                error.set(Some(format!("Building added but failed to reload list: {}", e)));
                            }
                        }
                        submitting.set(false);
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("You don't have permission to add buildings".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to add building: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let can_create = auth.is_admin_or_manager();

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    html! {
        <div class="container mt-4">
            <h2>{"Buildings"}</h2>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            {
                if can_create {
                    html! {
                        <form class="row g-2 mb-3" onsubmit={on_submit}>
                            <div class="col-md-6">
                                <input
                                    class="form-control"
                                    placeholder="Address"
                                    value={(*address).clone()}
                                    disabled={*submitting}
                                    oninput={{
                                        let address = address.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            address.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                            <div class="col-md-3">
                                <input
                                    class="form-control"
                                    placeholder="Year"
                                    type="number"
                                    value={(*year).clone()}
                                    disabled={*submitting}
                                    oninput={{
                                        let year = year.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            year.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                            <div class="col-md-3">
                                <button class="btn btn-primary" type="submit" disabled={*submitting}>
                                    if *submitting {
                                        <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                                    }
                                    {"Add Building"}
                                </button>
                            </div>
                        </form>
                    }
                } else {
                    html! {
                        <div class="alert alert-secondary small">
                            {"You don't have permission to add buildings."}
                        </div>
                    }
                }
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if (*buildings).is_empty() {
                <div class="alert alert-info">
                    {"No buildings found. "}
                    if can_create {
                        {"Add your first building above."}
                    }
                </div>
            } else {
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>{"ID"}</th>
                            <th>{"Address"}</th>
                            <th>{"Year"}</th>
                            <th>{"Actions"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for (*buildings).iter().map(|b| html!{
                            <tr>
                                <td>{b.id}</td>
                                <td>{b.address.clone()}</td>
                                <td>{b.construction_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into())}</td>
                                <td>
                                    <Link<Route> to={Route::BuildingApartments { id: b.id }} classes="btn btn-sm btn-secondary">
                                        {"View Apartments"}
                                    </Link<Route>>
                                </td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
