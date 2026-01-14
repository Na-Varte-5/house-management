use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::ErrorAlert;
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Building {
    id: u64,
    address: String,
    construction_year: Option<i32>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Apartment {
    id: u64,
    building_id: u64,
    number: String,
    size_sq_m: Option<f64>,
}

#[function_component(BuildingApartmentsPage)]
pub fn building_apartments_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let route = use_route::<Route>();
    let building_id = match route {
        Some(Route::BuildingApartments { id }) => id,
        _ => 0,
    };

    let building = use_state(|| None::<Building>);
    let apartments = use_state(|| Vec::<Apartment>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Load building details and user's apartments on mount or when building_id changes
    {
        let building = building.clone();
        let apartments = apartments.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = auth.token().map(|t| t.to_string());

        use_effect_with(building_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());

                // Load building details
                let building_result = client.get::<Building>(&format!("/buildings/{}", id)).await;

                // Load apartments
                let apartments_endpoint = format!("/buildings/{}/apartments/my", id);
                let apartments_result = client.get::<Vec<Apartment>>(&apartments_endpoint).await;

                match (building_result, apartments_result) {
                    (Ok(b), Ok(list)) => {
                        building.set(Some(b));
                        apartments.set(list);
                        loading.set(false);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(format!("Failed to load data: {}", e)));
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

    html! {
        <div class="container mt-4">
            <h3>
                {"Apartments in "}
                {
                    if let Some(ref b) = *building {
                        b.address.clone()
                    } else {
                        format!("Building {}", building_id)
                    }
                }
            </h3>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if (*apartments).is_empty() {
                <div class="alert alert-info">
                    {"No apartments found in this building that you have access to."}
                </div>
            } else {
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>{"ID"}</th>
                            <th>{"Number"}</th>
                            <th>{"Size (m²)"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for (*apartments).iter().map(|a| html!{
                            <tr>
                                <td>{a.id}</td>
                                <td>{a.number.clone()}</td>
                                <td>{
                                    a.size_sq_m
                                        .map(|s| format!("{:.2}", s))
                                        .unwrap_or_else(|| "-".into())
                                }</td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
