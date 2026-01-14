use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize)]
struct CreatedResponse {
    id: u64,
}

#[derive(Deserialize, Clone, PartialEq)]
struct ApartmentWithBuilding {
    id: u64,
    number: String,
    building_id: u64,
    building_address: String,
}

#[derive(Serialize)]
struct NewMaintenanceRequest {
    apartment_id: u64,
    request_type: String,
    priority: String,
    title: String,
    description: String,
}

#[function_component(MaintenanceNewPage)]
pub fn maintenance_new_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let apartments = use_state(|| Vec::<ApartmentWithBuilding>::new());
    let loading_apartments = use_state(|| true);

    let apartment_id = use_state(|| None::<u64>);
    let request_type = use_state(|| "General".to_string());
    let priority = use_state(|| "Medium".to_string());
    let title = use_state(String::default);
    let description = use_state(String::default);

    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Load user's apartments
    {
        let apartments = apartments.clone();
        let loading = loading_apartments.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<ApartmentWithBuilding>>("/apartments/my").await {
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
        let apartment_id = apartment_id.clone();
        let request_type = request_type.clone();
        let priority = priority.clone();
        let title = title.clone();
        let description = description.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let navigator = navigator.clone();
        let token = auth.token().map(|t| t.to_string());

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // Validation
            if apartment_id.is_none() {
                error.set(Some("Please select an apartment".to_string()));
                return;
            }
            if title.trim().is_empty() {
                error.set(Some("Title is required".to_string()));
                return;
            }
            if description.trim().is_empty() {
                error.set(Some("Description is required".to_string()));
                return;
            }

            let apartment_id = apartment_id.clone();
            let request_type = request_type.clone();
            let priority = priority.clone();
            let title = title.clone();
            let description = description.clone();
            let submitting = submitting.clone();
            let error = error.clone();
            let success = success.clone();
            let navigator = navigator.clone();
            let token = token.clone();

            submitting.set(true);
            error.set(None);
            success.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let new_request = NewMaintenanceRequest {
                    apartment_id: apartment_id.unwrap(),
                    request_type: (*request_type).clone(),
                    priority: (*priority).clone(),
                    title: (*title).clone(),
                    description: (*description).clone(),
                };

                match client.post::<_, CreatedResponse>("/requests", &new_request).await {
                    Ok(response) => {
                        success.set(Some("Request created successfully! Redirecting...".to_string()));
                        // Redirect to the created request's detail page
                        let request_id = response.id;
                        gloo_timers::callback::Timeout::new(1000, move || {
                            navigator.push(&Route::MaintenanceDetail { id: request_id });
                        }).forget();
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("You don't have permission to create requests".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create request: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Maintenance);
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

    html! {
        <div class="container mt-4">
            <div class="row justify-content-center">
                <div class="col-md-8 col-lg-6">
                    <div class="card">
                        <div class="card-header">
                            <h4 class="mb-0">{"New Maintenance Request"}</h4>
                        </div>
                        <div class="card-body">
                            if let Some(err) = (*error).clone() {
                                <ErrorAlert message={err} on_close={clear_error.clone()} />
                            }

                            if let Some(msg) = (*success).clone() {
                                <SuccessAlert message={msg} on_close={clear_success.clone()} />
                            }

                            <form onsubmit={on_submit}>
                                // Apartment selector
                                <div class="mb-3">
                                    <label class="form-label">{"Apartment"}<span class="text-danger">{"*"}</span></label>
                                    if *loading_apartments {
                                        <div class="text-muted small">{"Loading apartments..."}</div>
                                    } else {
                                        <select
                                            class="form-select"
                                            disabled={*submitting}
                                            onchange={{
                                                let apartment_id = apartment_id.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    let val = select.value();
                                                    if let Ok(id) = val.parse::<u64>() {
                                                        apartment_id.set(Some(id));
                                                    }
                                                })
                                            }}
                                        >
                                            <option value="">{"-- Select Apartment --"}</option>
                                            {
                                                for apartments.iter().map(|apt| {
                                                    html! {
                                                        <option value={apt.id.to_string()}>
                                                            {format!("Apartment {} - {}", apt.number, apt.building_address)}
                                                        </option>
                                                    }
                                                })
                                            }
                                        </select>
                                    }
                                </div>

                                // Request Type
                                <div class="mb-3">
                                    <label class="form-label">{"Request Type"}<span class="text-danger">{"*"}</span></label>
                                    <select
                                        class="form-select"
                                        disabled={*submitting}
                                        value={(*request_type).clone()}
                                        onchange={{
                                            let request_type = request_type.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                request_type.set(select.value());
                                            })
                                        }}
                                    >
                                        <option value="General">{"General"}</option>
                                        <option value="Plumbing">{"Plumbing"}</option>
                                        <option value="Electrical">{"Electrical"}</option>
                                        <option value="HVAC">{"HVAC"}</option>
                                        <option value="Appliance">{"Appliance"}</option>
                                        <option value="Structural">{"Structural"}</option>
                                        <option value="Other">{"Other"}</option>
                                    </select>
                                </div>

                                // Priority
                                <div class="mb-3">
                                    <label class="form-label">{"Priority"}<span class="text-danger">{"*"}</span></label>
                                    <select
                                        class="form-select"
                                        disabled={*submitting}
                                        value={(*priority).clone()}
                                        onchange={{
                                            let priority = priority.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                priority.set(select.value());
                                            })
                                        }}
                                    >
                                        <option value="Low" selected={*priority == "Low"}>{"Low"}</option>
                                        <option value="Medium" selected={*priority == "Medium"}>{"Medium"}</option>
                                        <option value="High" selected={*priority == "High"}>{"High"}</option>
                                        <option value="Urgent" selected={*priority == "Urgent"}>{"Urgent"}</option>
                                    </select>
                                </div>

                                // Title
                                <div class="mb-3">
                                    <label class="form-label">{"Title"}<span class="text-danger">{"*"}</span></label>
                                    <input
                                        type="text"
                                        class="form-control"
                                        placeholder="Brief description of the issue"
                                        disabled={*submitting}
                                        value={(*title).clone()}
                                        oninput={{
                                            let title = title.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                title.set(input.value());
                                            })
                                        }}
                                    />
                                </div>

                                // Description
                                <div class="mb-3">
                                    <label class="form-label">{"Description"}<span class="text-danger">{"*"}</span></label>
                                    <textarea
                                        class="form-control"
                                        rows="5"
                                        placeholder="Detailed description of the maintenance issue"
                                        disabled={*submitting}
                                        value={(*description).clone()}
                                        oninput={{
                                            let description = description.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                                description.set(textarea.value());
                                            })
                                        }}
                                    ></textarea>
                                </div>

                                // Buttons
                                <div class="d-flex justify-content-end gap-2">
                                    <button
                                        type="button"
                                        class="btn btn-secondary"
                                        disabled={*submitting}
                                        onclick={on_cancel}
                                    >
                                        {"Cancel"}
                                    </button>
                                    <button
                                        type="submit"
                                        class="btn btn-primary"
                                        disabled={*submitting}
                                    >
                                        if *submitting {
                                            <>
                                                <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                                                {"Creating..."}
                                            </>
                                        } else {
                                            {"Create Request"}
                                        }
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
