use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

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
}

#[function_component(MaintenanceListPage)]
pub fn maintenance_list_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let requests = use_state(|| Vec::<MaintenanceRequest>::new());
    let filter_status = use_state(|| "All".to_string());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Load requests on mount
    {
        let requests = requests.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<MaintenanceRequest>>("/requests").await {
                    Ok(list) => {
                        requests.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load requests: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_new_request = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MaintenanceNew);
        })
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    // Filter requests by status
    let filtered_requests: Vec<MaintenanceRequest> = if *filter_status == "All" {
        (*requests).clone()
    } else {
        requests.iter()
            .filter(|r| r.status == *filter_status)
            .cloned()
            .collect()
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
            <div class="d-flex justify-content-between align-items-center mb-3">
                <h2>{"Maintenance Requests"}</h2>
                <button class="btn btn-primary" onclick={on_new_request}>
                    {"+ New Request"}
                </button>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            // Filter buttons
            <div class="btn-group mb-3" role="group">
                {
                    for ["All", "Open", "InProgress", "Resolved"].iter().map(|status| {
                        let status_str = status.to_string();
                        let filter_status = filter_status.clone();
                        let is_active = *filter_status == status_str;
                        html! {
                            <button
                                type="button"
                                class={classes!("btn", "btn-sm", if is_active { "btn-primary" } else { "btn-outline-primary" })}
                                onclick={{
                                    let status_str = status_str.clone();
                                    Callback::from(move |_| filter_status.set(status_str.clone()))
                                }}
                            >
                                {status_str}
                            </button>
                        }
                    })
                }
            </div>

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if filtered_requests.is_empty() {
                <div class="alert alert-info">
                    {
                        if *filter_status == "All" {
                            "No maintenance requests found. Create your first request above."
                        } else {
                            "No requests with this status."
                        }
                    }
                </div>
            } else {
                <div class="row">
                    {
                        for filtered_requests.iter().map(|req| {
                            let id = req.id;
                            let navigator = navigator.clone();
                            html! {
                                <div class="col-md-6 col-lg-4 mb-3">
                                    <div
                                        class="card h-100 cursor-pointer"
                                        style="cursor: pointer;"
                                        onclick={{
                                            Callback::from(move |_| {
                                                navigator.push(&Route::MaintenanceDetail { id });
                                            })
                                        }}
                                    >
                                        <div class="card-body">
                                            <div class="d-flex justify-content-between align-items-start mb-2">
                                                <h5 class="card-title mb-0">{&req.title}</h5>
                                                <span class={classes!("badge", priority_class(&req.priority))}>
                                                    {&req.priority}
                                                </span>
                                            </div>
                                            <p class="card-text text-muted small mb-2">
                                                {&req.description}
                                            </p>
                                            <div class="d-flex justify-content-between align-items-center">
                                                <span class={classes!("badge", status_class(&req.status))}>
                                                    {&req.status}
                                                </span>
                                                <small class="text-muted">{&req.request_type}</small>
                                            </div>
                                            <small class="text-muted d-block mt-2">
                                                {"Apartment #"}{req.apartment_id}
                                            </small>
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
            }
        </div>
    }
}
