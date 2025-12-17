use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Apartment {
    id: u64,
    building_id: u64,
    number: String,
    size_sq_m: Option<f64>,
}

#[derive(Serialize)]
struct NewApartment {
    building_id: u64,
    number: String,
    size_sq_m: Option<f64>,
}

#[function_component(BuildingApartmentsPage)]
pub fn building_apartments_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let route = use_route::<Route>();
    let building_id = match route {
        Some(Route::BuildingApartments { id }) => id,
        _ => 0,
    };

    let apartments = use_state(|| Vec::<Apartment>::new());
    let number = use_state(String::default);
    let size = use_state(String::default);
    let loading = use_state(|| true);
    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Load apartments on mount or when building_id changes
    {
        let apartments = apartments.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with(building_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let endpoint = format!("/buildings/{}/apartments", id);

                match client.get::<Vec<Apartment>>(&endpoint).await {
                    Ok(list) => {
                        apartments.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load apartments: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_submit = {
        let number = number.clone();
        let size = size.clone();
        let apartments = apartments.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let number_val = (*number).clone();
            let size_val = (*size).clone();

            if number_val.is_empty() {
                error.set(Some("Apartment number is required".to_string()));
                return;
            }

            let apartments = apartments.clone();
            let number = number.clone();
            let size = size.clone();
            let submitting = submitting.clone();
            let error = error.clone();
            let success = success.clone();
            let token = token.clone();

            submitting.set(true);
            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_apartment = NewApartment {
                    building_id,
                    number: number_val,
                    size_sq_m: size_val.parse::<f64>().ok(),
                };

                match client.post::<_, serde_json::Value>("/apartments", &new_apartment).await {
                    Ok(_) => {
                        // Reload apartments list
                        let endpoint = format!("/buildings/{}/apartments", building_id);
                        match client.get::<Vec<Apartment>>(&endpoint).await {
                            Ok(list) => {
                                apartments.set(list);
                                number.set(String::new());
                                size.set(String::new());
                                success.set(Some("Apartment added successfully".to_string()));
                            }
                            Err(e) => {
                                error.set(Some(format!("Apartment added but failed to reload list: {}", e)));
                            }
                        }
                        submitting.set(false);
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("You don't have permission to add apartments".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to add apartment: {}", e)));
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
            <h3>{format!("Apartments in Building {}", building_id)}</h3>

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
                            <div class="col-md-4">
                                <input
                                    class="form-control"
                                    placeholder="Apartment Number"
                                    value={(*number).clone()}
                                    disabled={*submitting}
                                    oninput={{
                                        let number = number.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            number.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                            <div class="col-md-4">
                                <input
                                    class="form-control"
                                    placeholder="Size (m²)"
                                    type="number"
                                    step="0.1"
                                    value={(*size).clone()}
                                    disabled={*submitting}
                                    oninput={{
                                        let size = size.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            size.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                            <div class="col-md-4">
                                <button class="btn btn-primary" type="submit" disabled={*submitting}>
                                    if *submitting {
                                        <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                                    }
                                    {"Add Apartment"}
                                </button>
                            </div>
                        </form>
                    }
                } else {
                    html! {
                        <div class="alert alert-secondary small">
                            {"You don't have permission to add apartments."}
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
            } else if (*apartments).is_empty() {
                <div class="alert alert-info">
                    {"No apartments found in this building. "}
                    if can_create {
                        {"Add your first apartment above."}
                    }
                </div>
            } else {
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>{"ID"}</th>
                            <th>{"Number"}</th>
                            <th>{"Size (m²)"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for (*apartments).iter().map(|a| html!{
                            <tr>
                                <td>{a.id}</td>
                                <td>{a.number.clone()}</td>
                                <td>{
                                    a.size_sq_m
                                        .map(|s| format!("{:.2}", s))
                                        .unwrap_or_else(|| "-".into())
                                }</td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
