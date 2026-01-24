use super::types::UserInfo;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenterInfo {
    pub id: u64,
    pub user_id: u64,
    pub apartment_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub user_name: String,
    pub user_email: String,
}

#[derive(Properties, PartialEq)]
pub struct RenterManagementProps {
    pub renters: Vec<RenterInfo>,
    pub all_users: Vec<UserInfo>,
    pub user_query: String,
    pub on_query_change: Callback<String>,
    pub on_assign: Callback<(u64, Option<String>, Option<String>, bool)>, // (user_id, start_date, end_date, is_active)
    pub on_remove: Callback<u64>,                                         // user_id
    pub on_toggle_active: Callback<u64>,                                  // user_id
    pub loading: bool,
    pub show: bool,
}

#[function_component(RenterManagement)]
pub fn renter_management(props: &RenterManagementProps) -> Html {
    let start_date = use_state(|| Option::<String>::None);
    let end_date = use_state(|| Option::<String>::None);
    let is_active = use_state(|| true);

    if !props.show {
        return html! {
            <div class="alert alert-info small mb-0">
                <i class="bi bi-info-circle me-2"></i>
                {"Select an apartment to manage its renters"}
            </div>
        };
    }

    // Filter users based on query
    let filtered_users: Vec<&UserInfo> = if props.user_query.is_empty() {
        props.all_users.iter().collect()
    } else {
        let query_lower = props.user_query.to_lowercase();
        props
            .all_users
            .iter()
            .filter(|u| {
                u.name.to_lowercase().contains(&query_lower)
                    || u.email.to_lowercase().contains(&query_lower)
            })
            .collect()
    };

    // Exclude already assigned renters
    let renter_user_ids: Vec<u64> = props.renters.iter().map(|r| r.user_id).collect();
    let available_users: Vec<&UserInfo> = filtered_users
        .into_iter()
        .filter(|u| !renter_user_ids.contains(&u.id))
        .take(5)
        .collect();

    // Separate active and past renters
    let (active_renters, past_renters): (Vec<&RenterInfo>, Vec<&RenterInfo>) =
        props.renters.iter().partition(|r| r.is_active);

    html! {
        <div>
            <h6 class="small fw-semibold mb-2">{"Current Renters"}</h6>
            if props.loading {
                <div class="text-center py-2">
                    <div class="spinner-border spinner-border-sm" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if active_renters.is_empty() {
                <div class="alert alert-warning small mb-3">
                    {"No active renters for this apartment"}
                </div>
            } else {
                <ul class="list-group list-group-flush mb-3">
                    { for active_renters.iter().map(|renter| {
                        let user_id = renter.user_id;

                        let on_remove = {
                            let on_remove_cb = props.on_remove.clone();
                            Callback::from(move |_| on_remove_cb.emit(user_id))
                        };

                        let on_toggle = {
                            let on_toggle_cb = props.on_toggle_active.clone();
                            Callback::from(move |_| on_toggle_cb.emit(user_id))
                        };

                        html! {
                            <li class="list-group-item px-0">
                                <div class="d-flex justify-content-between align-items-start">
                                    <div class="flex-grow-1">
                                        <div class="d-flex align-items-center mb-1">
                                            <strong class="small me-2">{&renter.user_name}</strong>
                                            <span class="badge bg-success">{"Active"}</span>
                                        </div>
                                        <span class="text-muted" style="font-size: 0.75rem;">{&renter.user_email}</span>
                                        <div class="mt-1" style="font-size: 0.75rem;">
                                            if let Some(start) = &renter.start_date {
                                                <span class="text-muted">
                                                    {"Start: "}<strong>{start}</strong>
                                                </span>
                                            }
                                            if let Some(end) = &renter.end_date {
                                                <span class="text-muted ms-2">
                                                    {"End: "}<strong>{end}</strong>
                                                </span>
                                            }
                                        </div>
                                    </div>
                                    <div class="btn-group btn-group-sm" role="group">
                                        <button
                                            class="btn btn-sm btn-outline-secondary"
                                            onclick={on_toggle}
                                            title="Mark as inactive"
                                        >
                                            <i class="bi bi-pause-circle"></i>
                                        </button>
                                        <button
                                            class="btn btn-sm btn-outline-danger"
                                            onclick={on_remove}
                                            title="Remove renter"
                                        >
                                            <i class="bi bi-x-circle"></i>
                                        </button>
                                    </div>
                                </div>
                            </li>
                        }
                    }) }
                </ul>
            }

            if !past_renters.is_empty() {
                <h6 class="small fw-semibold mb-2 mt-3">{"Past Renters"}</h6>
                <ul class="list-group list-group-flush mb-3">
                    { for past_renters.iter().map(|renter| {
                        let user_id = renter.user_id;

                        let on_remove = {
                            let on_remove_cb = props.on_remove.clone();
                            Callback::from(move |_| on_remove_cb.emit(user_id))
                        };

                        let on_toggle = {
                            let on_toggle_cb = props.on_toggle_active.clone();
                            Callback::from(move |_| on_toggle_cb.emit(user_id))
                        };

                        html! {
                            <li class="list-group-item px-0">
                                <div class="d-flex justify-content-between align-items-start">
                                    <div class="flex-grow-1">
                                        <div class="d-flex align-items-center mb-1">
                                            <strong class="small me-2 text-muted">{&renter.user_name}</strong>
                                            <span class="badge bg-secondary">{"Inactive"}</span>
                                        </div>
                                        <span class="text-muted" style="font-size: 0.75rem;">{&renter.user_email}</span>
                                        <div class="mt-1" style="font-size: 0.75rem;">
                                            if let Some(start) = &renter.start_date {
                                                <span class="text-muted">
                                                    {"Start: "}<strong>{start}</strong>
                                                </span>
                                            }
                                            if let Some(end) = &renter.end_date {
                                                <span class="text-muted ms-2">
                                                    {"End: "}<strong>{end}</strong>
                                                </span>
                                            }
                                        </div>
                                    </div>
                                    <div class="btn-group btn-group-sm" role="group">
                                        <button
                                            class="btn btn-sm btn-outline-success"
                                            onclick={on_toggle}
                                            title="Mark as active"
                                        >
                                            <i class="bi bi-play-circle"></i>
                                        </button>
                                        <button
                                            class="btn btn-sm btn-outline-danger"
                                            onclick={on_remove}
                                            title="Remove renter"
                                        >
                                            <i class="bi bi-x-circle"></i>
                                        </button>
                                    </div>
                                </div>
                            </li>
                        }
                    }) }
                </ul>
            }

            <h6 class="small fw-semibold mb-2 mt-3">{"Assign New Renter"}</h6>

            <div class="row g-2 mb-2">
                <div class="col-md-6">
                    <label class="form-label small mb-1">{"Start Date"}</label>
                    <input
                        type="date"
                        class="form-control form-control-sm"
                        value={start_date.as_ref().map(|s| s.clone()).unwrap_or_default()}
                        oninput={{
                            let start_date = start_date.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                let value = input.value();
                                start_date.set(if value.is_empty() { None } else { Some(value) });
                            })
                        }}
                    />
                </div>
                <div class="col-md-6">
                    <label class="form-label small mb-1">{"End Date"}</label>
                    <input
                        type="date"
                        class="form-control form-control-sm"
                        value={end_date.as_ref().map(|s| s.clone()).unwrap_or_default()}
                        oninput={{
                            let end_date = end_date.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                let value = input.value();
                                end_date.set(if value.is_empty() { None } else { Some(value) });
                            })
                        }}
                    />
                </div>
            </div>

            <div class="form-check form-switch mb-2">
                <input
                    class="form-check-input"
                    type="checkbox"
                    id="renter-active-toggle"
                    checked={*is_active}
                    onchange={{
                        let is_active = is_active.clone();
                        Callback::from(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            is_active.set(input.checked());
                        })
                    }}
                />
                <label class="form-check-label small" for="renter-active-toggle">
                    {"Active rental"}
                </label>
            </div>

            <input
                type="text"
                class="form-control form-control-sm mb-2"
                placeholder="Search users..."
                value={props.user_query.clone()}
                oninput={{
                    let on_query_change = props.on_query_change.clone();
                    Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        on_query_change.emit(input.value());
                    })
                }}
            />

            if !props.user_query.is_empty() && !available_users.is_empty() {
                <ul class="list-group list-group-flush">
                    { for available_users.iter().map(|user| {
                        let user_id = user.id;
                        let start_date_val = (*start_date).clone();
                        let end_date_val = (*end_date).clone();
                        let is_active_val = *is_active;

                        let on_assign = {
                            let on_assign_cb = props.on_assign.clone();
                            Callback::from(move |_| {
                                on_assign_cb.emit((user_id, start_date_val.clone(), end_date_val.clone(), is_active_val))
                            })
                        };

                        html! {
                            <li
                                class="list-group-item list-group-item-action px-2"
                                onclick={on_assign}
                                style="cursor: pointer;"
                            >
                                <div class="small">
                                    <strong>{&user.name}</strong>
                                    <br/>
                                    <span class="text-muted" style="font-size: 0.75rem;">{&user.email}</span>
                                </div>
                            </li>
                        }
                    }) }
                </ul>
            } else if !props.user_query.is_empty() && available_users.is_empty() {
                <div class="alert alert-info small mb-0">
                    {"No matching users found"}
                </div>
            }
        </div>
    }
}
