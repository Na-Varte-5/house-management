use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::ErrorAlert;
use crate::components::search_input::SearchInput;
use crate::contexts::AuthContext;
use crate::i18n::{t, t_with_args};
use crate::routes::Route;
use crate::services::api_client;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Building {
    id: u64,
    address: String,
    construction_year: Option<i32>,
}

#[function_component(BuildingsPage)]
pub fn buildings_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let buildings = use_state(|| Vec::<Building>::new());
    let search_query = use_state(String::default);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    {
        let buildings = buildings.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings/my").await {
                    Ok(list) => {
                        buildings.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(t_with_args(
                            "buildings-failed-load",
                            &[("error", &e.to_string())],
                        )));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let on_search_change = {
        let search_query = search_query.clone();
        Callback::from(move |val: String| search_query.set(val))
    };

    let query_lower = search_query.to_lowercase();
    let filtered_buildings: Vec<&Building> = buildings
        .iter()
        .filter(|b| query_lower.is_empty() || b.address.to_lowercase().contains(&query_lower))
        .collect();

    let heading = if auth.is_admin_or_manager() {
        t("buildings-title-all")
    } else {
        t("buildings-title-my")
    };

    html! {
        <div class="container mt-4">
            <div class="d-flex justify-content-between align-items-center mb-4">
                <h2 class="mb-0">{heading}</h2>
                <SearchInput
                    value={(*search_query).clone()}
                    on_change={on_search_change}
                    placeholder={t("buildings-search-placeholder")}
                />
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            } else if filtered_buildings.is_empty() {
                if search_query.is_empty() {
                    <div class="text-center py-5">
                        <i class="bi bi-building text-muted" style="font-size: 4rem;"></i>
                        <h4 class="text-muted mt-3">{t("buildings-no-buildings")}</h4>
                        <p class="text-muted">
                            {t("buildings-no-buildings-desc")}
                        </p>
                    </div>
                } else {
                    <div class="alert alert-info">{t("buildings-no-match")}</div>
                }
            } else {
                <div class="row">
                    { for filtered_buildings.iter().map(|b| {
                        let building_id = b.id;
                        html! {
                            <div class="col-md-6 col-lg-4 mb-3" key={building_id}>
                                <Link<Route> to={Route::BuildingApartments { id: building_id }} classes="text-decoration-none">
                                    <div class="card h-100 border-0 shadow-sm">
                                        <div class="card-body">
                                            <div class="d-flex align-items-start">
                                                <div class="bg-primary bg-opacity-10 rounded-3 p-3 me-3">
                                                    <i class="bi bi-building text-primary" style="font-size: 1.5rem;"></i>
                                                </div>
                                                <div>
                                                    <h5 class="card-title text-dark mb-1">{&b.address}</h5>
                                                    {
                                                        if let Some(year) = b.construction_year {
                                                            html! {
                                                                <span class="text-muted small">
                                                                    <i class="bi bi-calendar3 me-1"></i>
                                                                    {t_with_args("buildings-built", &[("year", &year.to_string())])}
                                                                </span>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                        <div class="card-footer bg-transparent border-0 pt-0">
                                            <small class="text-primary">
                                                {t("buildings-view-apartments")} <i class="bi bi-arrow-right ms-1"></i>
                                            </small>
                                        </div>
                                    </div>
                                </Link<Route>>
                            </div>
                        }
                    }) }
                </div>
            }
        </div>
    }
}
