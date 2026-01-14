use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;
use crate::components::announcement_list::AnnouncementList;
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

#[derive(Deserialize, Clone)]
struct DashboardStats {
    open_maintenance_count: i64,
    active_proposals_count: i64,
    my_apartments_count: i64,
    pending_votes_count: i64,
    meters_due_calibration: i64,
}

#[function_component(Home)]
pub fn home() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();
    let is_authenticated = auth.is_authenticated();

    let stats = use_state(|| None::<DashboardStats>);
    let loading = use_state(|| false);

    // Load dashboard stats for authenticated users
    {
        let stats = stats.clone();
        let loading = loading.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with(is_authenticated, move |is_auth| {
            if *is_auth {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(data) = client.get::<DashboardStats>("/dashboard").await {
                        stats.set(Some(data));
                    }
                    loading.set(false);
                });
            }
            || ()
        });
    }

    if !is_authenticated {
        // Public view: show announcements
        return html! {
            <div class="container mt-4" style="padding-top: 56px;">
                <h1>{"Announcements"}</h1>
                <AnnouncementList />
            </div>
        };
    }

    // Authenticated dashboard
    html! {
        <div class="container-fluid">
            <h2 class="mb-4">{"Dashboard"}</h2>

            // Stats cards
            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border text-primary" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if let Some(data) = (*stats).clone() {
                <div class="row g-3 mb-4">
                    // Open Maintenance
                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-danger">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{"Open Maintenance"}</h6>
                                        <h2 class="card-title mb-0">{data.open_maintenance_count}</h2>
                                    </div>
                                    <i class="bi bi-tools" style="font-size: 3rem; opacity: 0.5;"></i>
                                </div>
                                <button
                                    class="btn btn-light btn-sm mt-3 w-100"
                                    onclick={{
                                        let nav = navigator.clone();
                                        Callback::from(move |_| nav.push(&Route::Maintenance))
                                    }}
                                >
                                    {"View Requests"}
                                </button>
                            </div>
                        </div>
                    </div>

                    // Active Proposals
                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-info">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{"Active Proposals"}</h6>
                                        <h2 class="card-title mb-0">{data.active_proposals_count}</h2>
                                    </div>
                                    <i class="bi bi-check2-square" style="font-size: 3rem; opacity: 0.5;"></i>
                                </div>
                                <button
                                    class="btn btn-light btn-sm mt-3 w-100"
                                    onclick={{
                                        let nav = navigator.clone();
                                        Callback::from(move |_| nav.push(&Route::Voting))
                                    }}
                                >
                                    {"View Proposals"}
                                </button>
                            </div>
                        </div>
                    </div>

                    // My Properties
                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-success">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{"My Properties"}</h6>
                                        <h2 class="card-title mb-0">{data.my_apartments_count}</h2>
                                    </div>
                                    <i class="bi bi-building" style="font-size: 3rem; opacity: 0.5;"></i>
                                </div>
                                <button
                                    class="btn btn-light btn-sm mt-3 w-100"
                                    onclick={{
                                        let nav = navigator.clone();
                                        Callback::from(move |_| nav.push(&Route::Buildings))
                                    }}
                                >
                                    {"View Buildings"}
                                </button>
                            </div>
                        </div>
                    </div>

                    // Pending Votes
                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-warning">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{"Pending Votes"}</h6>
                                        <h2 class="card-title mb-0">{data.pending_votes_count}</h2>
                                    </div>
                                    <i class="bi bi-hourglass-split" style="font-size: 3rem; opacity: 0.5;"></i>
                                </div>
                                <button
                                    class="btn btn-light btn-sm mt-3 w-100"
                                    onclick={{
                                        let nav = navigator.clone();
                                        Callback::from(move |_| nav.push(&Route::Voting))
                                    }}
                                >
                                    {"Vote Now"}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                // Additional info row
                if data.meters_due_calibration > 0 {
                    <div class="alert alert-warning d-flex align-items-center mb-4" role="alert">
                        <i class="bi bi-exclamation-triangle-fill me-2" style="font-size: 1.5rem;"></i>
                        <div>
                            <strong>{"Meter Calibration Due"}</strong>
                            <p class="mb-0">{format!("{} meter(s) need calibration within 30 days", data.meters_due_calibration)}</p>
                        </div>
                    </div>
                }
            }

            // Recent Announcements section
            <div class="row">
                <div class="col-12">
                    <div class="card">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <h5 class="mb-0">{"Recent Announcements"}</h5>
                        </div>
                        <div class="card-body">
                            <AnnouncementList />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
