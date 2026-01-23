use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{ErrorAlert, SuccessAlert, TextInput, Textarea, Select, SelectOption, DateTimeInput, Checkbox, FormGroup};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Serialize)]
struct CreateProposalPayload {
    title: String,
    description: String,
    building_id: Option<u64>,
    start_time: String,
    end_time: String,
    voting_method: String,
    eligible_roles: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct Building {
    id: u64,
    address: String,
}

// Helper function to get current datetime as string
fn now_datetime() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year() as i32;
    let month = (now.get_month() as f64 + 1.0) as i32;
    let day = now.get_date() as i32;
    let hours = now.get_hours() as i32;
    let minutes = now.get_minutes() as i32;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
}

// Helper function to add days to current datetime
fn datetime_plus_days(days: f64) -> String {
    let now = js_sys::Date::new_0();
    now.set_date((now.get_date() as f64 + days) as u32);
    let year = now.get_full_year() as i32;
    let month = (now.get_month() as f64 + 1.0) as i32;
    let day = now.get_date() as i32;
    let hours = now.get_hours() as i32;
    let minutes = now.get_minutes() as i32;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
}

#[function_component(VotingNewPage)]
pub fn voting_new_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    {"Only Admins and Managers can create proposals."}
                </div>
            </div>
        };
    }

    let buildings = use_state(|| Vec::<Building>::new());
    let selected_building = use_state(|| "".to_string());

    let title = use_state(String::default);
    let description = use_state(String::default);
    let start_time = use_state(now_datetime);
    let end_time = use_state(|| datetime_plus_days(7.0));
    let voting_method = use_state(|| "SimpleMajority".to_string());

    let role_admin = use_state(|| false);
    let role_manager = use_state(|| false);
    let role_homeowner = use_state(|| true);
    let role_renter = use_state(|| false);
    let role_hoa = use_state(|| false);

    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    // Load user's accessible buildings on mount
    {
        let buildings = buildings.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<Building>>("/buildings/my").await {
                    buildings.set(list);
                }
            });
            || ()
        });
    }

    let on_submit = {
        let title = title.clone();
        let description = description.clone();
        let selected_building = selected_building.clone();
        let start_time = start_time.clone();
        let end_time = end_time.clone();
        let voting_method = voting_method.clone();
        let role_admin = role_admin.clone();
        let role_manager = role_manager.clone();
        let role_homeowner = role_homeowner.clone();
        let role_renter = role_renter.clone();
        let role_hoa = role_hoa.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let navigator = navigator.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // Validation
            if title.trim().is_empty() {
                error.set(Some("Title is required".to_string()));
                return;
            }
            if description.trim().is_empty() {
                error.set(Some("Description is required".to_string()));
                return;
            }
            if start_time.is_empty() {
                error.set(Some("Start time is required".to_string()));
                return;
            }
            if end_time.is_empty() {
                error.set(Some("End time is required".to_string()));
                return;
            }

            let mut eligible_roles = Vec::new();
            if *role_admin {
                eligible_roles.push("Admin".to_string());
            }
            if *role_manager {
                eligible_roles.push("Manager".to_string());
            }
            if *role_homeowner {
                eligible_roles.push("Homeowner".to_string());
            }
            if *role_renter {
                eligible_roles.push("Renter".to_string());
            }
            if *role_hoa {
                eligible_roles.push("HOA Member".to_string());
            }

            if eligible_roles.is_empty() {
                error.set(Some("At least one role must be eligible".to_string()));
                return;
            }

            let title = title.clone();
            let description = description.clone();
            let selected_building = selected_building.clone();
            let start_time = start_time.clone();
            let end_time = end_time.clone();
            let voting_method = voting_method.clone();
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

                // Parse building_id from string
                let building_id = if selected_building.is_empty() {
                    None
                } else {
                    selected_building.parse().ok()
                };

                let payload = CreateProposalPayload {
                    title: (*title).clone(),
                    description: (*description).clone(),
                    building_id,
                    start_time: (*start_time).clone(),
                    end_time: (*end_time).clone(),
                    voting_method: (*voting_method).clone(),
                    eligible_roles,
                };

                match client.post::<_, serde_json::Value>("/proposals", &payload).await {
                    Ok(_) => {
                        success.set(Some("Proposal created successfully".to_string()));
                        gloo_timers::callback::Timeout::new(1500, move || {
                            navigator.push(&Route::Voting);
                        }).forget();
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("You don't have permission to create proposals".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create proposal: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Voting);
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
    let on_title_change = {
        let title = title.clone();
        Callback::from(move |value: String| title.set(value))
    };

    let on_description_change = {
        let description = description.clone();
        Callback::from(move |value: String| description.set(value))
    };

    let on_building_change = {
        let selected_building = selected_building.clone();
        Callback::from(move |value: String| selected_building.set(value))
    };

    let on_method_change = {
        let voting_method = voting_method.clone();
        Callback::from(move |value: String| voting_method.set(value))
    };

    let on_start_change = {
        let start_time = start_time.clone();
        Callback::from(move |value: String| start_time.set(value))
    };

    let on_end_change = {
        let end_time = end_time.clone();
        Callback::from(move |value: String| end_time.set(value))
    };

    let on_admin_change = {
        let role_admin = role_admin.clone();
        Callback::from(move |checked: bool| role_admin.set(checked))
    };

    let on_manager_change = {
        let role_manager = role_manager.clone();
        Callback::from(move |checked: bool| role_manager.set(checked))
    };

    let on_homeowner_change = {
        let role_homeowner = role_homeowner.clone();
        Callback::from(move |checked: bool| role_homeowner.set(checked))
    };

    let on_renter_change = {
        let role_renter = role_renter.clone();
        Callback::from(move |checked: bool| role_renter.set(checked))
    };

    let on_hoa_change = {
        let role_hoa = role_hoa.clone();
        Callback::from(move |checked: bool| role_hoa.set(checked))
    };

    // Build building options for Select component
    let building_options = {
        let mut options = vec![SelectOption::new("", "Global (visible to all buildings)")];
        for building in buildings.iter() {
            options.push(SelectOption::new(building.id.to_string(), &building.address));
        }
        options
    };

    // Build voting method options
    let method_options = vec![
        SelectOption::new("SimpleMajority", "Simple Majority (1 person = 1 vote)"),
        SelectOption::new("WeightedArea", "Weighted by Area (vote weight = apartment size)"),
        SelectOption::new("PerSeat", "Per Seat (1 apartment = 1 vote)"),
        SelectOption::new("Consensus", "Consensus (no 'No' votes allowed)"),
    ];

    html! {
        <div class="container mt-4">
            <div class="row justify-content-center">
                <div class="col-md-8 col-lg-6">
                    <div class="card">
                        <div class="card-header">
                            <h4 class="mb-0">{"Create New Proposal"}</h4>
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
                                    title="Basic Information"
                                    description="Enter the proposal details"
                                >
                                    <TextInput
                                        label="Title"
                                        value={(*title).clone()}
                                        on_change={on_title_change}
                                        placeholder="Proposal title"
                                        disabled={*submitting}
                                        required=true
                                    />

                                    <Textarea
                                        label="Description"
                                        value={(*description).clone()}
                                        on_change={on_description_change}
                                        placeholder="Describe the proposal"
                                        rows={4}
                                        disabled={*submitting}
                                        required=true
                                    />

                                    <Select
                                        label="Building Scope"
                                        value={(*selected_building).clone()}
                                        on_change={on_building_change}
                                        options={building_options}
                                        disabled={*submitting}
                                        help_text="Leave as Global to make this proposal visible to all users, or select a building to restrict visibility"
                                    />
                                </FormGroup>

                                <FormGroup title="Voting Settings">
                                    <Select
                                        label="Voting Method"
                                        value={(*voting_method).clone()}
                                        on_change={on_method_change}
                                        options={method_options}
                                        disabled={*submitting}
                                        required=true
                                    />

                                    <DateTimeInput
                                        label="Voting Start Time"
                                        value={(*start_time).clone()}
                                        on_change={on_start_change}
                                        input_type="datetime-local"
                                        disabled={*submitting}
                                        required=true
                                    />

                                    <DateTimeInput
                                        label="Voting End Time"
                                        value={(*end_time).clone()}
                                        on_change={on_end_change}
                                        input_type="datetime-local"
                                        min={Some((*start_time).clone())}
                                        disabled={*submitting}
                                        required=true
                                    />
                                </FormGroup>

                                <FormGroup
                                    title="Eligible Voters"
                                    description="Select which roles can vote on this proposal"
                                >
                                    <Checkbox
                                        id="role-admin"
                                        label="Admins"
                                        checked={*role_admin}
                                        on_change={on_admin_change}
                                        disabled={*submitting}
                                    />

                                    <Checkbox
                                        id="role-manager"
                                        label="Managers"
                                        checked={*role_manager}
                                        on_change={on_manager_change}
                                        disabled={*submitting}
                                    />

                                    <Checkbox
                                        id="role-homeowner"
                                        label="Homeowners"
                                        checked={*role_homeowner}
                                        on_change={on_homeowner_change}
                                        disabled={*submitting}
                                    />

                                    <Checkbox
                                        id="role-renter"
                                        label="Renters"
                                        checked={*role_renter}
                                        on_change={on_renter_change}
                                        disabled={*submitting}
                                    />

                                    <Checkbox
                                        id="role-hoa"
                                        label="HOA Members"
                                        checked={*role_hoa}
                                        on_change={on_hoa_change}
                                        disabled={*submitting}
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
                                            {"Create Proposal"}
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
