use crate::components::ErrorAlert;
use crate::components::pagination::Pagination;
use crate::components::search_input::SearchInput;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::routes::Route;
use crate::services::api::{PaginatedResponse, PaginationMeta, api_client};
use serde::Deserialize;
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct MaintenanceRequest {
    id: u64,
    apartment_id: u64,
    apartment_number: String,
    building_id: u64,
    building_address: String,
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
    let pagination_meta = use_state(|| None::<PaginationMeta>);
    let current_page = use_state(|| 1i64);
    let filter_status = use_state(|| "All".to_string());
    let search_query = use_state(String::default);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    {
        let requests = requests.clone();
        let pagination_meta = pagination_meta.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());
        let page = *current_page;

        use_effect_with(page, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let client = api_client(token.as_deref());
                let url = format!("/requests?page={}&per_page=20", page);
                match client
                    .get::<PaginatedResponse<MaintenanceRequest>>(&url)
                    .await
                {
                    Ok(resp) => {
                        pagination_meta.set(Some(resp.pagination));
                        requests.set(resp.data);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("{}: {}", t("error-load-failed"), e)));
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

    let on_page_change = {
        let current_page = current_page.clone();
        Callback::from(move |page: i64| {
            current_page.set(page);
        })
    };

    let on_search_change = {
        let search_query = search_query.clone();
        Callback::from(move |val: String| search_query.set(val))
    };

    let query_lower = search_query.to_lowercase();
    let filtered_requests: Vec<MaintenanceRequest> = requests
        .iter()
        .filter(|r| *filter_status == "All" || r.status == *filter_status)
        .filter(|r| {
            query_lower.is_empty()
                || r.title.to_lowercase().contains(&query_lower)
                || r.description.to_lowercase().contains(&query_lower)
                || r.apartment_number.to_lowercase().contains(&query_lower)
                || r.building_address.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect();

    let mut grouped_requests: HashMap<u64, (String, Vec<MaintenanceRequest>)> = HashMap::new();
    for req in filtered_requests.iter() {
        grouped_requests
            .entry(req.building_id)
            .or_insert_with(|| (req.building_address.clone(), Vec::new()))
            .1
            .push(req.clone());
    }

    let mut building_groups: Vec<(u64, String, Vec<MaintenanceRequest>)> = grouped_requests
        .into_iter()
        .map(|(id, (addr, reqs))| (id, addr, reqs))
        .collect();
    building_groups.sort_by(|a, b| a.1.cmp(&b.1));

    let priority_class = |priority: &str| match priority {
        "Urgent" => "bg-danger",
        "High" => "bg-warning text-dark",
        "Medium" => "bg-info",
        "Low" => "bg-secondary",
        _ => "bg-light text-dark",
    };

    let status_class = |status: &str| match status {
        "Open" => "bg-primary",
        "InProgress" => "bg-warning text-dark",
        "Resolved" => "bg-success",
        _ => "bg-secondary",
    };

    let meta = (*pagination_meta).clone();

    html! {
        <div class="container mt-4">
            <div class="d-flex justify-content-between align-items-center mb-3">
                <h2>{t("maintenance-title")}</h2>
                <button class="btn btn-primary" onclick={on_new_request}>
                    {t("maintenance-new-request")}
                </button>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            <div class="d-flex flex-wrap gap-2 align-items-center mb-3">
                <div class="btn-group" role="group">
                    {
                    for ["All", "Open", "InProgress", "Resolved"].iter().map(|status| {
                        let status_str = status.to_string();
                        let filter_status = filter_status.clone();
                        let is_active = *filter_status == status_str;
                        let display = match *status {
                            "All" => t("maintenance-status-all"),
                            "Open" => t("maintenance-status-open"),
                            "InProgress" => t("maintenance-status-in-progress"),
                            "Resolved" => t("maintenance-status-resolved"),
                            _ => status_str.clone(),
                        };
                        html! {
                            <button
                                type="button"
                                class={classes!("btn", "btn-sm", if is_active { "btn-primary" } else { "btn-outline-primary" })}
                                onclick={{
                                    let status_str = status_str.clone();
                                    Callback::from(move |_| filter_status.set(status_str.clone()))
                                }}
                            >
                                {display}
                            </button>
                        }
                    })
                }
                </div>
                <SearchInput
                    value={(*search_query).clone()}
                    on_change={on_search_change}
                    placeholder={t("maintenance-search-placeholder")}
                />
            </div>

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            } else if filtered_requests.is_empty() {
                <div class="alert alert-info">
                    {
                        if *filter_status == "All" {
                            t("maintenance-no-requests")
                        } else {
                            t("maintenance-no-requests-status")
                        }
                    }
                </div>
            } else {
                <div>
                    {
                        for building_groups.iter().map(|(building_id, building_address, reqs)| {
                            html! {
                                <div class="mb-4" key={*building_id}>
                                    <h4 class="mb-3 text-secondary">{building_address}</h4>
                                    <div class="row">
                                        {
                                            for reqs.iter().map(|req| {
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
                                                                    {t("label-apartment")}{" "}{&req.apartment_number}
                                                                </small>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                        }
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>

                if let Some(ref m) = meta {
                    <Pagination
                        current_page={m.page}
                        total_pages={m.total_pages}
                        total_items={m.total}
                        on_page_change={on_page_change.clone()}
                    />
                }
            }
        </div>
    }
}
