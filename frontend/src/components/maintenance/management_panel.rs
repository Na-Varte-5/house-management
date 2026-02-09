use crate::i18n::{t, t_with_args};
use crate::services::{ApiError, api_client};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct MaintenanceRequest {
    pub id: u64,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<u64>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
struct UpdateRequest {
    status: Option<String>,
    priority: Option<String>,
    assigned_to: Option<u64>,
}

#[derive(Properties, PartialEq)]
pub struct ManagementPanelProps {
    pub request: MaintenanceRequest,
    pub users: Vec<UserInfo>,
    pub token: Option<String>,
    pub on_update: Callback<()>,
    pub on_error: Callback<String>,
    pub on_success: Callback<String>,
}

/// Management panel for updating request status, priority, and assignment
/// Only displayed for Admin/Manager roles
#[function_component(ManagementPanel)]
pub fn management_panel(props: &ManagementPanelProps) -> Html {
    let updating = use_state(|| false);
    let new_status = use_state(|| None::<String>);
    let new_priority = use_state(|| None::<String>);
    let new_assigned = use_state(|| None::<u64>);

    let on_update_status = {
        let request_id = props.request.id;
        let new_status = new_status.clone();
        let updating = updating.clone();
        let on_error = props.on_error.clone();
        let on_success = props.on_success.clone();
        let on_update = props.on_update.clone();
        let token = props.token.clone();

        Callback::from(move |_| {
            if let Some(status) = (*new_status).clone() {
                let updating = updating.clone();
                let on_error = on_error.clone();
                let on_success = on_success.clone();
                let on_update = on_update.clone();
                let token = token.clone();
                let new_status = new_status.clone();

                updating.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: Some(status),
                        priority: None,
                        assigned_to: None,
                    };

                    match client
                        .put::<_, serde_json::Value>(&format!("/requests/{}", request_id), &update)
                        .await
                    {
                        Ok(_) => {
                            on_success.emit(t("maintenance-status-updated"));
                            on_update.emit(());
                            new_status.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            on_error.emit(t("maintenance-no-permission-update"));
                        }
                        Err(e) => {
                            on_error.emit(t_with_args(
                                "maintenance-failed-update-status",
                                &[("error", &e.to_string())],
                            ));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    let on_update_priority = {
        let request_id = props.request.id;
        let new_priority = new_priority.clone();
        let updating = updating.clone();
        let on_error = props.on_error.clone();
        let on_success = props.on_success.clone();
        let on_update = props.on_update.clone();
        let token = props.token.clone();

        Callback::from(move |_| {
            if let Some(priority) = (*new_priority).clone() {
                let updating = updating.clone();
                let on_error = on_error.clone();
                let on_success = on_success.clone();
                let on_update = on_update.clone();
                let token = token.clone();
                let new_priority = new_priority.clone();

                updating.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: None,
                        priority: Some(priority),
                        assigned_to: None,
                    };

                    match client
                        .put::<_, serde_json::Value>(&format!("/requests/{}", request_id), &update)
                        .await
                    {
                        Ok(_) => {
                            on_success.emit(t("maintenance-priority-updated"));
                            on_update.emit(());
                            new_priority.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            on_error.emit(t("maintenance-no-permission-update"));
                        }
                        Err(e) => {
                            on_error.emit(t_with_args(
                                "maintenance-failed-update-priority",
                                &[("error", &e.to_string())],
                            ));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    let on_assign = {
        let request_id = props.request.id;
        let new_assigned = new_assigned.clone();
        let updating = updating.clone();
        let on_error = props.on_error.clone();
        let on_success = props.on_success.clone();
        let on_update = props.on_update.clone();
        let token = props.token.clone();

        Callback::from(move |_| {
            if let Some(user_id) = *new_assigned {
                let updating = updating.clone();
                let on_error = on_error.clone();
                let on_success = on_success.clone();
                let on_update = on_update.clone();
                let token = token.clone();
                let new_assigned = new_assigned.clone();

                updating.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    let update = UpdateRequest {
                        status: None,
                        priority: None,
                        assigned_to: Some(user_id),
                    };

                    match client
                        .put::<_, serde_json::Value>(&format!("/requests/{}", request_id), &update)
                        .await
                    {
                        Ok(_) => {
                            on_success.emit(t("maintenance-assigned-success"));
                            on_update.emit(());
                            new_assigned.set(None);
                        }
                        Err(ApiError::Forbidden) => {
                            on_error.emit(t("maintenance-no-permission-assign"));
                        }
                        Err(e) => {
                            on_error.emit(t_with_args(
                                "maintenance-failed-assign",
                                &[("error", &e.to_string())],
                            ));
                        }
                    }
                    updating.set(false);
                });
            }
        })
    };

    html! {
        <div class="card">
            <div class="card-header">
                <h5 class="mb-0">{t("maintenance-management")}</h5>
            </div>
            <div class="card-body">
                // Update Status
                <div class="mb-3">
                    <label class="form-label small fw-semibold">{t("maintenance-update-status")}</label>
                    <select
                        class="form-select form-select-sm mb-2"
                        disabled={*updating}
                        value={(*new_status).clone().unwrap_or_else(|| props.request.status.clone())}
                        onchange={{
                            let new_status = new_status.clone();
                            Callback::from(move |e: Event| {
                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                let value = select.value();
                                if !value.is_empty() {
                                    new_status.set(Some(value));
                                }
                            })
                        }}
                    >
                        <option value="Open" selected={props.request.status == "Open"}>{t("maintenance-status-open")}</option>
                        <option value="InProgress" selected={props.request.status == "InProgress"}>{t("maintenance-status-in-progress")}</option>
                        <option value="Resolved" selected={props.request.status == "Resolved"}>{t("maintenance-status-resolved")}</option>
                    </select>
                    <button
                        class="btn btn-sm btn-primary w-100"
                        disabled={new_status.is_none() || *updating}
                        onclick={on_update_status}
                    >
                        if *updating {
                            <span class="spinner-border spinner-border-sm me-1"></span>
                        }
                        {t("maintenance-update-status-btn")}
                    </button>
                </div>

                <hr />

                // Update Priority
                <div class="mb-3">
                    <label class="form-label small fw-semibold">{t("maintenance-update-priority")}</label>
                    <select
                        class="form-select form-select-sm mb-2"
                        disabled={*updating}
                        value={(*new_priority).clone().unwrap_or_else(|| props.request.priority.clone())}
                        onchange={{
                            let new_priority = new_priority.clone();
                            Callback::from(move |e: Event| {
                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                let value = select.value();
                                if !value.is_empty() {
                                    new_priority.set(Some(value));
                                }
                            })
                        }}
                    >
                        <option value="Low" selected={props.request.priority == "Low"}>{t("maintenance-priority-low")}</option>
                        <option value="Medium" selected={props.request.priority == "Medium"}>{t("maintenance-priority-medium")}</option>
                        <option value="High" selected={props.request.priority == "High"}>{t("maintenance-priority-high")}</option>
                        <option value="Urgent" selected={props.request.priority == "Urgent"}>{t("maintenance-priority-urgent")}</option>
                    </select>
                    <button
                        class="btn btn-sm btn-primary w-100"
                        disabled={new_priority.is_none() || *updating}
                        onclick={on_update_priority}
                    >
                        if *updating {
                            <span class="spinner-border spinner-border-sm me-1"></span>
                        }
                        {t("maintenance-update-priority-btn")}
                    </button>
                </div>

                <hr />

                // Assign User
                <div class="mb-3">
                    <label class="form-label small fw-semibold">{t("maintenance-assign-to")}</label>
                    <select
                        class="form-select form-select-sm mb-2"
                        disabled={*updating}
                        value={
                            new_assigned.map(|id| id.to_string())
                                .or_else(|| props.request.assigned_to.map(|id| id.to_string()))
                                .unwrap_or_default()
                        }
                        onchange={{
                            let new_assigned = new_assigned.clone();
                            Callback::from(move |e: Event| {
                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                let value = select.value();
                                if !value.is_empty() {
                                    if let Ok(id) = value.parse::<u64>() {
                                        new_assigned.set(Some(id));
                                    }
                                }
                            })
                        }}
                    >
                        <option value="">{t("maintenance-unassigned-option")}</option>
                        {
                            for props.users.iter().map(|user| {
                                let is_selected = props.request.assigned_to == Some(user.id);
                                html! {
                                    <option value={user.id.to_string()} selected={is_selected}>
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
                        {t("maintenance-assign-request-btn")}
                    </button>
                </div>
            </div>
        </div>
    }
}
