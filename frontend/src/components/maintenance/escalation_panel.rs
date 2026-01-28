use crate::services::api_client;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct Manager {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Properties, PartialEq)]
pub struct EscalationPanelProps {
    pub request_id: u64,
    pub building_id: u64,
    pub token: Option<String>,
    pub on_escalated: Callback<()>,
    pub on_error: Callback<String>,
}

#[function_component(EscalationPanel)]
pub fn escalation_panel(props: &EscalationPanelProps) -> Html {
    let managers = use_state(|| Vec::<Manager>::new());
    let loading_managers = use_state(|| false);
    let escalating = use_state(|| false);
    let show_panel = use_state(|| false);
    let selected_manager = use_state(|| None::<u64>);

    let on_show_panel = {
        let show_panel = show_panel.clone();
        let loading_managers = loading_managers.clone();
        let managers = managers.clone();
        let token = props.token.clone();
        let building_id = props.building_id;

        Callback::from(move |_: MouseEvent| {
            show_panel.set(true);
            loading_managers.set(true);

            let managers = managers.clone();
            let loading_managers = loading_managers.clone();
            let token = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<Vec<Manager>>(&format!("/buildings/{}/managers", building_id))
                    .await
                {
                    Ok(list) => managers.set(list),
                    Err(_) => managers.set(vec![]),
                }
                loading_managers.set(false);
            });
        })
    };

    let on_cancel = {
        let show_panel = show_panel.clone();
        let selected_manager = selected_manager.clone();
        Callback::from(move |_: MouseEvent| {
            show_panel.set(false);
            selected_manager.set(None);
        })
    };

    let on_select_manager = {
        let selected_manager = selected_manager.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
            if let Some(select) = target {
                let value = select.value();
                if value.is_empty() {
                    selected_manager.set(None);
                } else if let Ok(id) = value.parse::<u64>() {
                    selected_manager.set(Some(id));
                }
            }
        })
    };

    let on_escalate = {
        let escalating = escalating.clone();
        let show_panel = show_panel.clone();
        let selected_manager = selected_manager.clone();
        let token = props.token.clone();
        let request_id = props.request_id;
        let on_escalated = props.on_escalated.clone();
        let on_error = props.on_error.clone();

        Callback::from(move |_: MouseEvent| {
            if let Some(manager_id) = *selected_manager {
                let escalating = escalating.clone();
                let show_panel = show_panel.clone();
                let selected_manager = selected_manager.clone();
                let token = token.clone();
                let on_escalated = on_escalated.clone();
                let on_error = on_error.clone();

                escalating.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());

                    #[derive(Serialize)]
                    struct EscalatePayload {
                        manager_id: u64,
                    }

                    match client
                        .post::<EscalatePayload, serde_json::Value>(
                            &format!("/requests/{}/escalate", request_id),
                            &EscalatePayload { manager_id },
                        )
                        .await
                    {
                        Ok(_) => {
                            show_panel.set(false);
                            selected_manager.set(None);
                            on_escalated.emit(());
                        }
                        Err(e) => {
                            on_error.emit(format!("Failed to escalate: {}", e));
                        }
                    }
                    escalating.set(false);
                });
            }
        })
    };

    html! {
        <div class="card mt-3">
            <div class="card-header">
                <h5 class="mb-0">{"Escalate Request"}</h5>
            </div>
            <div class="card-body">
                if !*show_panel {
                    <p class="text-muted small mb-2">
                        {"Need help from a building manager? Escalate this request for faster resolution."}
                    </p>
                    <button
                        class="btn btn-warning btn-sm"
                        onclick={on_show_panel}
                    >
                        <i class="bi bi-arrow-up-circle me-1"></i>
                        {"Escalate to Manager"}
                    </button>
                } else {
                    if *loading_managers {
                        <div class="text-center py-2">
                            <div class="spinner-border spinner-border-sm" role="status"></div>
                            <span class="ms-2">{"Loading managers..."}</span>
                        </div>
                    } else if managers.is_empty() {
                        <div class="alert alert-info mb-2">
                            {"No managers assigned to this building. Please contact administration."}
                        </div>
                        <button class="btn btn-secondary btn-sm" onclick={on_cancel}>
                            {"Cancel"}
                        </button>
                    } else {
                        <div class="mb-3">
                            <label class="form-label">{"Select a manager"}</label>
                            <select
                                class="form-select"
                                onchange={on_select_manager}
                            >
                                <option value="">{"-- Select Manager --"}</option>
                                {
                                    for managers.iter().map(|m| {
                                        html! {
                                            <option value={m.id.to_string()}>
                                                {format!("{} ({})", m.name, m.email)}
                                            </option>
                                        }
                                    })
                                }
                            </select>
                        </div>
                        <div class="d-flex gap-2">
                            <button
                                class="btn btn-warning btn-sm"
                                onclick={on_escalate}
                                disabled={selected_manager.is_none() || *escalating}
                            >
                                if *escalating {
                                    <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                                    {"Escalating..."}
                                } else {
                                    {"Confirm Escalation"}
                                }
                            </button>
                            <button
                                class="btn btn-secondary btn-sm"
                                onclick={on_cancel}
                                disabled={*escalating}
                            >
                                {"Cancel"}
                            </button>
                        </div>
                    }
                }
            </div>
        </div>
    }
}
