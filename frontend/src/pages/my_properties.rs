use crate::components::ErrorAlert;
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct UserProperty {
    id: u64,
    apartment_number: String,
    building_id: u64,
    building_address: String,
    size_sq_m: Option<f64>,
    bedrooms: Option<i32>,
    bathrooms: Option<i32>,
    relationship: String, // "owner" or "renter"
    is_active: bool,
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct PropertyStats {
    total_properties: usize,
    active_maintenance_requests: i64,
    pending_votes: i64,
}

#[derive(Deserialize, Clone, PartialEq)]
struct MyPropertiesResponse {
    properties: Vec<UserProperty>,
    stats: PropertyStats,
}

#[function_component(MyProperties)]
pub fn my_properties() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();
    let data = use_state(|| None::<MyPropertiesResponse>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    // Load data on mount
    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<MyPropertiesResponse>("/users/me/properties")
                    .await
                {
                    Ok(response) => data.set(Some(response)),
                    Err(e) => error.set(Some(format!("Failed to load properties: {}", e))),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Render loading state
    if *loading {
        return html! {
            <div class="container mt-4">
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            </div>
        };
    }

    // Render error state
    if let Some(err) = (*error).clone() {
        let on_close = {
            let error = error.clone();
            Callback::from(move |_| error.set(None))
        };

        return html! {
            <div class="container mt-4">
                <ErrorAlert message={err} on_close={on_close} />
            </div>
        };
    }

    // Render data
    let response = match (*data).clone() {
        Some(r) => r,
        None => {
            return html! {
                <div class="container mt-4">
                    <div class="alert alert-warning">{"No data available"}</div>
                </div>
            };
        }
    };

    html! {
        <div class="container mt-4">
            <h1 class="mb-4">{"My Properties"}</h1>

            // Statistics cards
            <div class="row mb-4">
                <div class="col-md-4 mb-3">
                    <div class="card">
                        <div class="card-body">
                            <h5 class="card-title text-muted">{"Total Properties"}</h5>
                            <h2 class="mb-0">{response.stats.total_properties}</h2>
                        </div>
                    </div>
                </div>
                <div class="col-md-4 mb-3">
                    <div class="card">
                        <div class="card-body">
                            <h5 class="card-title text-muted">{"Active Maintenance Requests"}</h5>
                            <h2 class="mb-0">{response.stats.active_maintenance_requests}</h2>
                        </div>
                    </div>
                </div>
                <div class="col-md-4 mb-3">
                    <div class="card">
                        <div class="card-body">
                            <h5 class="card-title text-muted">{"Pending Votes"}</h5>
                            <h2 class="mb-0">{response.stats.pending_votes}</h2>
                        </div>
                    </div>
                </div>
            </div>

            // Properties list
            <div class="card">
                <div class="card-header">
                    <h4 class="mb-0">{"Your Properties"}</h4>
                </div>
                <div class="card-body">
                    {
                        if response.properties.is_empty() {
                            html! {
                                <div class="alert alert-info">
                                    {"You don't have any properties yet. Contact your building administrator to get access."}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="row">
                                    { for response.properties.iter().map(|prop| render_property_card(prop, &navigator)) }
                                </div>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}

fn render_property_card(property: &UserProperty, navigator: &Navigator) -> Html {
    let on_click = {
        let navigator = navigator.clone();
        let apt_id = property.id;
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::ApartmentMeters {
                apartment_id: apt_id,
            });
        })
    };

    html! {
        <div class="col-md-6 col-lg-4 mb-3">
            <div class="card h-100" style="cursor: pointer;" onclick={on_click}>
                <div class="card-body">
                    <div class="d-flex justify-content-between align-items-start mb-2">
                        <h5 class="card-title mb-0">
                            {"Apartment "}{&property.apartment_number}
                        </h5>
                        <span class={format!("badge {}",
                            if property.relationship == "owner" {
                                "bg-primary"
                            } else if property.is_active {
                                "bg-success"
                            } else {
                                "bg-secondary"
                            }
                        )}>
                            {if property.relationship == "owner" {
                                "Owner"
                            } else if property.is_active {
                                "Active Renter"
                            } else {
                                "Past Renter"
                            }}
                        </span>
                    </div>

                    <p class="text-muted mb-2">
                        <i class="bi bi-building me-1"></i>
                        {&property.building_address}
                    </p>

                    {
                        if let Some(size) = property.size_sq_m {
                            html! {
                                <p class="mb-1">
                                    <i class="bi bi-arrows-angle-expand me-1"></i>
                                    {format!("{:.1} m²", size)}
                                </p>
                            }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if property.bedrooms.is_some() || property.bathrooms.is_some() {
                            html! {
                                <p class="mb-1">
                                    {
                                        if let Some(beds) = property.bedrooms {
                                            html! {
                                                <>
                                                    <i class="bi bi-door-closed me-1"></i>
                                                    {format!("{} bed", beds)}
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if property.bedrooms.is_some() && property.bathrooms.is_some() {
                                            html! { <span class="mx-1">{"•"}</span> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if let Some(baths) = property.bathrooms {
                                            html! {
                                                <>
                                                    <i class="bi bi-droplet me-1"></i>
                                                    {format!("{} bath", baths)}
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </p>
                            }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if property.relationship == "renter" {
                            html! {
                                <div class="mt-2 pt-2 border-top">
                                    {
                                        if let Some(start) = &property.start_date {
                                            html! {
                                                <small class="text-muted d-block">
                                                    {"Start: "}{start}
                                                </small>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if let Some(end) = &property.end_date {
                                            html! {
                                                <small class="text-muted d-block">
                                                    {"End: "}{end}
                                                </small>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                <div class="card-footer bg-transparent">
                    <small class="text-muted">{"Click to view details"}</small>
                </div>
            </div>
        </div>
    }
}
