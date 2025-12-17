use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};
use web_sys::HtmlInputElement;

#[derive(Deserialize, Clone, PartialEq)]
struct MaintenanceRequest {
    id: u64,
    apartment_id: u64,
    request_type: String,
    priority: String,
    title: String,
    description: String,
    status: String,
    created_by: u64,
    assigned_to: Option<u64>,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize, Clone, PartialEq)]
struct HistoryEntry {
    id: u64,
    request_id: u64,
    from_status: Option<String>,
    to_status: String,
    note: Option<String>,
    changed_by: u64,
    changed_at: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct Attachment {
    id: u64,
    filename: String,
    uploaded_by: u64,
    uploaded_at: String,
    url: String,
}

#[derive(Deserialize, Clone, PartialEq)]
struct UserInfo {
    id: u64,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UpdateRequest {
    status: Option<String>,
    priority: Option<String>,
    assigned_to: Option<u64>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: u64,
}

#[function_component(MaintenanceDetailPage)]
pub fn maintenance_detail_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let request = use_state(|| None::<MaintenanceRequest>);
    let history = use_state(|| Vec::<HistoryEntry>::new());
    let attachments = use_state(|| Vec::<Attachment>::new());
    let users = use_state(|| Vec::<UserInfo>::new());

    let loading = use_state(|| true);
    let loading_history = use_state(|| false);
    let loading_attachments = use_state(|| false);
    let updating = use_state(|| false);

    let new_status = use_state(|| None::<String>);
    let new_priority = use_state(|| None::<String>);
    let new_assigned = use_state(|| None::<u64>);

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let request_id = props.id;
    let token = auth.token().map(|t| t.to_string());

    // Load request details
    {
        let request = request.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<MaintenanceRequest>(&format!("/requests/{}", id)).await {
                    Ok(req) => {
                        request.set(Some(req));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load request: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    // Load history
    {
        let history = history.clone();
        let loading_history = loading_history.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            loading_history.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", id)).await {
                    history.set(list);
                }
                loading_history.set(false);
            });
            || ()
        });
    }

    // Load attachments
    {
        let attachments = attachments.clone();
        let loading_attachments = loading_attachments.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            loading_attachments.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<Attachment>>(&format!("/requests/{}/attachments", id)).await {
                    attachments.set(list);
                }
                loading_attachments.set(false);
            });
            || ()
        });
    }

    // Load users for assignment (Admin/Manager only)
    {
        let users = users.clone();
        let token = token.clone();
        let is_admin_or_manager = auth.is_admin_or_manager();

        use_effect_with((), move |_| {
            if is_admin_or_manager {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(list) = client.get::<Vec<UserInfo>>("/users/public").await {
                        users.set(list);
                    }
                });
            }
            || ()
        });
    }

    let on_update_status = {
        let request = request.clone();
        let new_status = new_status.clone();
        let updating = updating.clone();
        let error = error.clone();
        let success = success.clone();
        let history = history.clone();
        let token = token.clone();

        Callback::from(move |_| {
            if let (Some(req), Some(status)) = ((*request).clone(), (*new_status).clone()) {
                let request = request.clone();
                let history = history.clone();
                let updating = updating.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();
                let new_status = new_status.clone();

                updating.set(true);
                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: Some(status.clone()),
                        priority: None,
                        assigned_to: None,
                    };

                    match client.put::<_, MaintenanceRequest>(&format!("/requests/{}", req.id), &update).await {
                        Ok(updated) => {
                            request.set(Some(updated));
                            // Reload history
                            if let Ok(list) = client.get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", req.id)).await {
                                history.set(list);
                            }
                            success.set(Some("Status updated successfully".to_string()));
                            new_status.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some("You don't have permission to update requests".to_string()));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to update status: {}", e)));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    let on_update_priority = {
        let request = request.clone();
        let new_priority = new_priority.clone();
        let updating = updating.clone();
        let error = error.clone();
        let success = success.clone();
        let history = history.clone();
        let token = token.clone();

        Callback::from(move |_| {
            if let (Some(req), Some(priority)) = ((*request).clone(), (*new_priority).clone()) {
                let request = request.clone();
                let history = history.clone();
                let updating = updating.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();
                let new_priority = new_priority.clone();

                updating.set(true);
                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: None,
                        priority: Some(priority.clone()),
                        assigned_to: None,
                    };

                    match client.put::<_, MaintenanceRequest>(&format!("/requests/{}", req.id), &update).await {
                        Ok(updated) => {
                            request.set(Some(updated));
                            // Reload history
                            if let Ok(list) = client.get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", req.id)).await {
                                history.set(list);
                            }
                            success.set(Some("Priority updated successfully".to_string()));
                            new_priority.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some("You don't have permission to update requests".to_string()));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to update priority: {}", e)));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    let on_assign = {
        let request = request.clone();
        let new_assigned = new_assigned.clone();
        let updating = updating.clone();
        let error = error.clone();
        let success = success.clone();
        let history = history.clone();
        let token = token.clone();

        Callback::from(move |_| {
            if let (Some(req), Some(user_id)) = ((*request).clone(), *new_assigned) {
                let request = request.clone();
                let history = history.clone();
                let updating = updating.clone();
                let error = error.clone();
                let success = success.clone();
                let token = token.clone();
                let new_assigned = new_assigned.clone();

                updating.set(true);
                error.set(None);
                success.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: None,
                        priority: None,
                        assigned_to: Some(user_id),
                    };

                    match client.put::<_, MaintenanceRequest>(&format!("/requests/{}", req.id), &update).await {
                        Ok(updated) => {
                            request.set(Some(updated));
                            // Reload history
                            if let Ok(list) = client.get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", req.id)).await {
                                history.set(list);
                            }
                            success.set(Some("Request assigned successfully".to_string()));
                            new_assigned.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            error.set(Some("You don't have permission to assign requests".to_string()));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to assign request: {}", e)));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    let on_back = {
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

    // Priority badge color
    let priority_class = |priority: &str| match priority {
        "Urgent" => "bg-danger",
        "High" => "bg-warning text-dark",
        "Medium" => "bg-info",
        "Low" => "bg-secondary",
        _ => "bg-light text-dark",
    };

    // Status badge color
    let status_class = |status: &str| match status {
        "Open" => "bg-primary",
        "InProgress" => "bg-warning text-dark",
        "Resolved" => "bg-success",
        _ => "bg-secondary",
    };

    html! {
        <div class="container mt-4">
            <div class="mb-3">
                <button class="btn btn-outline-secondary btn-sm" onclick={on_back}>
                    {"← Back to List"}
                </button>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if let Some(req) = (*request).clone() {
                <div class="row">
                    // Main details
                    <div class="col-lg-8 mb-3">
                        <div class="card">
                            <div class="card-header">
                                <h4 class="mb-0">{&req.title}</h4>
                            </div>
                            <div class="card-body">
                                <div class="mb-3">
                                    <div class="d-flex gap-2 mb-2">
                                        <span class={classes!("badge", status_class(&req.status))}>
                                            {&req.status}
                                        </span>
                                        <span class={classes!("badge", priority_class(&req.priority))}>
                                            {&req.priority}
                                        </span>
                                        <span class="badge bg-light text-dark">
                                            {&req.request_type}
                                        </span>
                                    </div>
                                </div>

                                <div class="mb-3">
                                    <h6 class="text-muted small">{"Description"}</h6>
                                    <p class="mb-0">{&req.description}</p>
                                </div>

                                <hr />

                                <div class="row small text-muted">
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Apartment:"}</strong>{" "}{req.apartment_id}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Created by:"}</strong>{" "}{req.created_by}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Assigned to:"}</strong>{" "}
                                        {req.assigned_to.map(|id| id.to_string()).unwrap_or_else(|| "Unassigned".to_string())}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Created:"}</strong>{" "}{&req.created_at}
                                    </div>
                                </div>
                            </div>
                        </div>

                        // History
                        <div class="card mt-3">
                            <div class="card-header">
                                <h5 class="mb-0">{"History"}</h5>
                            </div>
                            <div class="card-body">
                                if *loading_history {
                                    <div class="text-center">
                                        <div class="spinner-border spinner-border-sm" role="status"></div>
                                    </div>
                                } else if history.is_empty() {
                                    <p class="text-muted small mb-0">{"No history available"}</p>
                                } else {
                                    <div class="timeline">
                                        {
                                            for history.iter().map(|entry| {
                                                html! {
                                                    <div class="mb-3 pb-3 border-bottom">
                                                        <div class="d-flex justify-content-between">
                                                            <strong class="small">{"User #"}{entry.changed_by}</strong>
                                                            <span class="small text-muted">
                                                                {entry.changed_at.as_ref().unwrap_or(&"(unknown)".to_string())}
                                                            </span>
                                                        </div>
                                                        <p class="mb-0 small">
                                                            {"Changed status from "}
                                                            <span class="text-decoration-line-through">
                                                                {entry.from_status.as_ref().unwrap_or(&"(none)".to_string())}
                                                            </span>
                                                            {" to "}
                                                            <strong>{&entry.to_status}</strong>
                                                            {
                                                                if let Some(note) = &entry.note {
                                                                    html! { <span class="text-muted">{" - "}{note}</span> }
                                                                } else {
                                                                    html! {}
                                                                }
                                                            }
                                                        </p>
                                                    </div>
                                                }
                                            })
                                        }
                                    </div>
                                }
                            </div>
                        </div>

                        // Attachments
                        <div class="card mt-3">
                            <div class="card-header">
                                <h5 class="mb-0">{"Attachments"}</h5>
                            </div>
                            <div class="card-body">
                                if *loading_attachments {
                                    <div class="text-center">
                                        <div class="spinner-border spinner-border-sm" role="status"></div>
                                    </div>
                                } else if attachments.is_empty() {
                                    <p class="text-muted small mb-0">{"No attachments"}</p>
                                } else {
                                    <div class="row">
                                        {
                                            for attachments.iter().map(|att| {
                                                html! {
                                                    <div class="col-md-4 mb-2">
                                                        <div class="card">
                                                            <div class="card-body p-2">
                                                                <p class="mb-1 small"><strong>{&att.filename}</strong></p>
                                                                <p class="mb-0 text-muted" style="font-size: 0.75rem;">
                                                                    {"Uploaded "}{&att.uploaded_at}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                        }
                                    </div>
                                }
                            </div>
                        </div>
                    </div>

                    // Admin controls
                    if auth.is_admin_or_manager() {
                        <div class="col-lg-4 mb-3">
                            <div class="card">
                                <div class="card-header">
                                    <h5 class="mb-0">{"Management"}</h5>
                                </div>
                                <div class="card-body">
                                    // Update Status
                                    <div class="mb-3">
                                        <label class="form-label small fw-semibold">{"Update Status"}</label>
                                        <select
                                            class="form-select form-select-sm mb-2"
                                            disabled={*updating}
                                            onchange={{
                                                let new_status = new_status.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    new_status.set(Some(select.value()));
                                                })
                                            }}
                                        >
                                            <option value="">{"-- Select Status --"}</option>
                                            <option value="Open">{"Open"}</option>
                                            <option value="InProgress">{"In Progress"}</option>
                                            <option value="Resolved">{"Resolved"}</option>
                                        </select>
                                        <button
                                            class="btn btn-sm btn-primary w-100"
                                            disabled={new_status.is_none() || *updating}
                                            onclick={on_update_status}
                                        >
                                            if *updating {
                                                <span class="spinner-border spinner-border-sm me-1"></span>
                                            }
                                            {"Update Status"}
                                        </button>
                                    </div>

                                    <hr />

                                    // Update Priority
                                    <div class="mb-3">
                                        <label class="form-label small fw-semibold">{"Update Priority"}</label>
                                        <select
                                            class="form-select form-select-sm mb-2"
                                            disabled={*updating}
                                            onchange={{
                                                let new_priority = new_priority.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    new_priority.set(Some(select.value()));
                                                })
                                            }}
                                        >
                                            <option value="">{"-- Select Priority --"}</option>
                                            <option value="Low">{"Low"}</option>
                                            <option value="Medium">{"Medium"}</option>
                                            <option value="High">{"High"}</option>
                                            <option value="Urgent">{"Urgent"}</option>
                                        </select>
                                        <button
                                            class="btn btn-sm btn-primary w-100"
                                            disabled={new_priority.is_none() || *updating}
                                            onclick={on_update_priority}
                                        >
                                            if *updating {
                                                <span class="spinner-border spinner-border-sm me-1"></span>
                                            }
                                            {"Update Priority"}
                                        </button>
                                    </div>

                                    <hr />

                                    // Assign User
                                    <div class="mb-3">
                                        <label class="form-label small fw-semibold">{"Assign To"}</label>
                                        <select
                                            class="form-select form-select-sm mb-2"
                                            disabled={*updating}
                                            onchange={{
                                                let new_assigned = new_assigned.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    if let Ok(id) = select.value().parse::<u64>() {
                                                        new_assigned.set(Some(id));
                                                    }
                                                })
                                            }}
                                        >
                                            <option value="">{"-- Select User --"}</option>
                                            {
                                                for users.iter().map(|user| {
                                                    html! {
                                                        <option value={user.id.to_string()}>
                                                            {format!("{} ({})", user.name, user.email)}
                                                        </option>
                                                    }
                                                })
                                            }
                                        </select>
                                        <button
                                            class="btn btn-sm btn-primary w-100"
                                            disabled={new_assigned.is_none() || *updating}
                                            onclick={on_assign}
                                        >
                                            if *updating {
                                                <span class="spinner-border spinner-border-sm me-1"></span>
                                            }
                                            {"Assign Request"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                </div>
            } else {
                <div class="alert alert-warning">
                    {"Request not found"}
                </div>
            }
        </div>
    }
}
