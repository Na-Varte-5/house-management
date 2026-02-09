use crate::components::ErrorAlert;
use crate::components::announcement_list::AnnouncementList;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

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
    let error = use_state(|| None::<String>);

    {
        let stats = stats.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with(is_authenticated, move |is_auth| {
            if *is_auth {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client.get::<DashboardStats>("/dashboard").await {
                        Ok(data) => stats.set(Some(data)),
                        Err(e) => error.set(Some(format!("{}: {}", t("error-load-failed"), e))),
                    }
                    loading.set(false);
                });
            }
            || ()
        });
    }

    if !is_authenticated {
        return html! {
            <div class="container mt-4" style="padding-top: 56px;">
                <div class="text-center py-5">
                    <i class="bi bi-house-door text-primary" style="font-size: 4rem;"></i>
                    <h1 class="mt-3">{t("home-hero-title")}</h1>
                    <p class="text-muted lead">{t("home-hero-subtitle")}</p>
                    <Link<Route> to={Route::Login} classes="btn btn-primary btn-lg mt-2">
                        {t("home-sign-in-button")}
                    </Link<Route>>
                </div>
                <hr class="my-4" />
                <h3 class="mb-3">{t("dashboard-recent-announcements")}</h3>
                <AnnouncementList />
            </div>
        };
    }

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    html! {
        <div class="container-fluid">
            <h2 class="mb-4">{t("dashboard-title")}</h2>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border text-primary" role="status">
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            } else if let Some(data) = (*stats).clone() {
                <div class="row g-3 mb-4">
                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-danger">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{t("dashboard-open-maintenance")}</h6>
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
                                    {t("dashboard-view-requests")}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-info">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{t("dashboard-active-proposals")}</h6>
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
                                    {t("dashboard-view-proposals")}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-success">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{t("dashboard-my-properties")}</h6>
                                        <h2 class="card-title mb-0">{data.my_apartments_count}</h2>
                                    </div>
                                    <i class="bi bi-building" style="font-size: 3rem; opacity: 0.5;"></i>
                                </div>
                                <button
                                    class="btn btn-light btn-sm mt-3 w-100"
                                    onclick={{
                                        let nav = navigator.clone();
                                        Callback::from(move |_| nav.push(&Route::MyProperties))
                                    }}
                                >
                                    {t("dashboard-view-properties")}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="col-md-6 col-lg-3">
                        <div class="card text-white bg-warning">
                            <div class="card-body">
                                <div class="d-flex justify-content-between align-items-center">
                                    <div>
                                        <h6 class="card-subtitle mb-2 text-white-50">{t("dashboard-pending-votes")}</h6>
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
                                    {t("dashboard-vote-now")}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                if data.meters_due_calibration > 0 {
                    <div class="alert alert-warning d-flex align-items-center mb-4" role="alert">
                        <i class="bi bi-exclamation-triangle-fill me-2" style="font-size: 1.5rem;"></i>
                        <div>
                            <strong>{t("dashboard-meter-calibration-due")}</strong>
                            <p class="mb-0">{format!("{} meter(s) need calibration within 30 days", data.meters_due_calibration)}</p>
                        </div>
                    </div>
                }

                if data.my_apartments_count == 0 {
                    <div class="alert alert-info d-flex align-items-center mb-4" role="alert">
                        <i class="bi bi-info-circle-fill me-2" style="font-size: 1.5rem;"></i>
                        <div>
                            <strong>{t("dashboard-getting-started")}</strong>
                            <p class="mb-0">{t("dashboard-no-properties")}</p>
                        </div>
                    </div>
                }
            }

            <div class="row">
                <div class="col-12">
                    <div class="card">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <h5 class="mb-0">{t("dashboard-recent-announcements")}</h5>
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
