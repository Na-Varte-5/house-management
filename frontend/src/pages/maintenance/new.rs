use crate::components::{
    ErrorAlert, FormGroup, Select, SelectOption, SuccessAlert, TextInput, Textarea,
};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{ApiError, api_client};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

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

    let apartment_id = use_state(|| "".to_string());
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
                match client
                    .get::<Vec<ApartmentWithBuilding>>("/apartments/my")
                    .await
                {
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
            if apartment_id.is_empty() {
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

                // Parse apartment_id
                let apt_id = match apartment_id.parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => {
                        error.set(Some("Invalid apartment selected".to_string()));
                        submitting.set(false);
                        return;
                    }
                };

                let new_request = NewMaintenanceRequest {
                    apartment_id: apt_id,
                    request_type: (*request_type).clone(),
                    priority: (*priority).clone(),
                    title: (*title).clone(),
                    description: (*description).clone(),
                };

                match client
                    .post::<_, CreatedResponse>("/requests", &new_request)
                    .await
                {
                    Ok(response) => {
                        success.set(Some(
                            "Request created successfully! Redirecting...".to_string(),
                        ));
                        // Redirect to the created request's detail page
                        let request_id = response.id;
                        gloo_timers::callback::Timeout::new(1000, move || {
                            navigator.push(&Route::MaintenanceDetail { id: request_id });
                        })
                        .forget();
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some(
                            "You don't have permission to create requests".to_string(),
                        ));
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

    // Callbacks for form inputs
    let on_apartment_change = {
        let apartment_id = apartment_id.clone();
        Callback::from(move |value: String| apartment_id.set(value))
    };

    let on_type_change = {
        let request_type = request_type.clone();
        Callback::from(move |value: String| request_type.set(value))
    };

    let on_priority_change = {
        let priority = priority.clone();
        Callback::from(move |value: String| priority.set(value))
    };

    let on_title_change = {
        let title = title.clone();
        Callback::from(move |value: String| title.set(value))
    };

    let on_description_change = {
        let description = description.clone();
        Callback::from(move |value: String| description.set(value))
    };

    // Build apartment options
    let apartment_options = {
        let mut options = vec![SelectOption::new("", "-- Select Apartment --")];
        for apt in apartments.iter() {
            let label = format!("Apartment {} - {}", apt.number, apt.building_address);
            options.push(SelectOption::new(apt.id.to_string(), label));
        }
        options
    };

    // Build request type options
    let type_options = vec![
        SelectOption::new("General", "General"),
        SelectOption::new("Plumbing", "Plumbing"),
        SelectOption::new("Electrical", "Electrical"),
        SelectOption::new("HVAC", "HVAC"),
        SelectOption::new("Appliance", "Appliance"),
        SelectOption::new("Structural", "Structural"),
        SelectOption::new("Other", "Other"),
    ];

    // Build priority options
    let priority_options = vec![
        SelectOption::new("Low", "Low"),
        SelectOption::new("Medium", "Medium"),
        SelectOption::new("High", "High"),
        SelectOption::new("Urgent", "Urgent"),
    ];

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
                                <FormGroup
                                    title="Request Details"
                                    description="Provide information about the maintenance issue"
                                >
                                    if *loading_apartments {
                                        <div class="mb-3">
                                            <label class="form-label">
                                                {"Apartment"}
                                                <span class="text-danger">{" *"}</span>
                                            </label>
                                            <div class="text-muted small">{"Loading apartments..."}</div>
                                        </div>
                                    } else {
                                        <Select
                                            label="Apartment"
                                            value={(*apartment_id).clone()}
                                            on_change={on_apartment_change}
                                            options={apartment_options}
                                            disabled={*submitting}
                                            required=true
                                            help_text="Select the apartment with the maintenance issue"
                                        />
                                    }

                                    <Select
                                        label="Request Type"
                                        value={(*request_type).clone()}
                                        on_change={on_type_change}
                                        options={type_options}
                                        disabled={*submitting}
                                        required=true
                                        help_text="Category of the maintenance issue"
                                    />

                                    <Select
                                        label="Priority"
                                        value={(*priority).clone()}
                                        on_change={on_priority_change}
                                        options={priority_options}
                                        disabled={*submitting}
                                        required=true
                                        help_text="How urgent is this issue?"
                                    />

                                    <TextInput
                                        label="Title"
                                        value={(*title).clone()}
                                        on_change={on_title_change}
                                        placeholder="Brief description of the issue"
                                        disabled={*submitting}
                                        required=true
                                    />

                                    <Textarea
                                        label="Description"
                                        value={(*description).clone()}
                                        on_change={on_description_change}
                                        placeholder="Detailed description of the maintenance issue"
                                        rows={5}
                                        disabled={*submitting}
                                        required=true
                                        help_text="Include any relevant details that will help us address the issue"
                                    />
                                </FormGroup>

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
