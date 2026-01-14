use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{ErrorAlert, SuccessAlert};
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
    let selected_building = use_state(|| None::<u64>);

    let title = use_state(String::default);
    let description = use_state(String::default);

    // Set date defaults: start_time = now, end_time = now + 7 days
    let start_time = use_state(|| {
        let now = js_sys::Date::new_0();
        let year = now.get_full_year() as i32;
        let month = (now.get_month() as f64 + 1.0) as i32;
        let day = now.get_date() as i32;
        let hours = now.get_hours() as i32;
        let minutes = now.get_minutes() as i32;
        format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
    });

    let end_time = use_state(|| {
        let now = js_sys::Date::new_0();
        // Add 7 days
        now.set_date((now.get_date() as f64 + 7.0) as u32);
        let year = now.get_full_year() as i32;
        let month = (now.get_month() as f64 + 1.0) as i32;
        let day = now.get_date() as i32;
        let hours = now.get_hours() as i32;
        let minutes = now.get_minutes() as i32;
        format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
    });

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
                let payload = CreateProposalPayload {
                    title: (*title).clone(),
                    description: (*description).clone(),
                    building_id: *selected_building,
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
                                // Title
                                <div class="mb-3">
                                    <label class="form-label">{"Title"}<span class="text-danger">{"*"}</span></label>
                                    <input
                                        type="text"
                                        class="form-control"
                                        placeholder="Proposal title"
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
                                        rows="4"
                                        placeholder="Describe the proposal"
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

                                // Building Scope
                                <div class="mb-3">
                                    <label class="form-label">
                                        {"Building Scope"}
                                        <span class="text-muted small ms-1">{"(optional)"}</span>
                                    </label>
                                    <select
                                        class="form-select"
                                        disabled={*submitting}
                                        value={selected_building.as_ref().map(|id| id.to_string()).unwrap_or_default()}
                                        onchange={{
                                            let selected_building = selected_building.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                let value = select.value();
                                                if value.is_empty() {
                                                    selected_building.set(None);
                                                } else {
                                                    selected_building.set(value.parse().ok());
                                                }
                                            })
                                        }}
                                    >
                                        <option value="">{"Global (visible to all buildings)"}</option>
                                        {for buildings.iter().map(|b| html! {
                                            <option value={b.id.to_string()}>{&b.address}</option>
                                        })}
                                    </select>
                                    <div class="form-text">{"Leave as Global to make this proposal visible to all users, or select a building to restrict visibility"}</div>
                                </div>

                                // Voting Method
                                <div class="mb-3">
                                    <label class="form-label">{"Voting Method"}<span class="text-danger">{"*"}</span></label>
                                    <select
                                        class="form-select"
                                        disabled={*submitting}
                                        value={(*voting_method).clone()}
                                        onchange={{
                                            let voting_method = voting_method.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                voting_method.set(select.value());
                                            })
                                        }}
                                    >
                                        <option value="SimpleMajority">{"Simple Majority (1 person = 1 vote)"}</option>
                                        <option value="WeightedArea">{"Weighted by Area (vote weight = apartment size)"}</option>
                                        <option value="PerSeat">{"Per Seat (1 apartment = 1 vote)"}</option>
                                        <option value="Consensus">{"Consensus (no 'No' votes allowed)"}</option>
                                    </select>
                                </div>

                                // Start Time
                                <div class="mb-3">
                                    <label class="form-label">{"Voting Start Time"}<span class="text-danger">{"*"}</span></label>
                                    <input
                                        type="datetime-local"
                                        class="form-control"
                                        disabled={*submitting}
                                        value={(*start_time).clone()}
                                        oninput={{
                                            let start_time = start_time.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                start_time.set(input.value());
                                            })
                                        }}
                                    />
                                </div>

                                // End Time
                                <div class="mb-3">
                                    <label class="form-label">{"Voting End Time"}<span class="text-danger">{"*"}</span></label>
                                    <input
                                        type="datetime-local"
                                        class="form-control"
                                        disabled={*submitting}
                                        value={(*end_time).clone()}
                                        oninput={{
                                            let end_time = end_time.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                end_time.set(input.value());
                                            })
                                        }}
                                    />
                                </div>

                                // Eligible Roles
                                <div class="mb-3">
                                    <label class="form-label">{"Eligible Voters"}<span class="text-danger">{"*"}</span></label>
                                    <div class="form-check">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id="admin"
                                            disabled={*submitting}
                                            checked={*role_admin}
                                            onchange={{
                                                let role_admin = role_admin.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    role_admin.set(input.checked());
                                                })
                                            }}
                                        />
                                        <label class="form-check-label" for="admin">
                                            {"Admins"}
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id="manager"
                                            disabled={*submitting}
                                            checked={*role_manager}
                                            onchange={{
                                                let role_manager = role_manager.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    role_manager.set(input.checked());
                                                })
                                            }}
                                        />
                                        <label class="form-check-label" for="manager">
                                            {"Managers"}
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id="homeowner"
                                            disabled={*submitting}
                                            checked={*role_homeowner}
                                            onchange={{
                                                let role_homeowner = role_homeowner.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    role_homeowner.set(input.checked());
                                                })
                                            }}
                                        />
                                        <label class="form-check-label" for="homeowner">
                                            {"Homeowners"}
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id="renter"
                                            disabled={*submitting}
                                            checked={*role_renter}
                                            onchange={{
                                                let role_renter = role_renter.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    role_renter.set(input.checked());
                                                })
                                            }}
                                        />
                                        <label class="form-check-label" for="renter">
                                            {"Renters"}
                                        </label>
                                    </div>
                                    <div class="form-check">
                                        <input
                                            class="form-check-input"
                                            type="checkbox"
                                            id="hoa"
                                            disabled={*submitting}
                                            checked={*role_hoa}
                                            onchange={{
                                                let role_hoa = role_hoa.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    role_hoa.set(input.checked());
                                                })
                                            }}
                                        />
                                        <label class="form-check-label" for="hoa">
                                            {"HOA Members"}
                                        </label>
                                    </div>
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
