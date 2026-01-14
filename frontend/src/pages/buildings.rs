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

#[function_component(BuildingsPage)]
pub fn buildings_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    let buildings = use_state(|| Vec::<Building>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Load user's buildings on mount
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
                        error.set(Some(format!("Failed to load buildings: {}", e)));
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
            <h2>{"Buildings"}</h2>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if (*buildings).is_empty() {
                <div class="alert alert-info">
                    {"No buildings found. You don't have any apartments assigned yet."}
                </div>
            } else {
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>{"ID"}</th>
                            <th>{"Address"}</th>
                            <th>{"Year"}</th>
                            <th>{"Actions"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for (*buildings).iter().map(|b| html!{
                            <tr>
                                <td>{b.id}</td>
                                <td>{b.address.clone()}</td>
                                <td>{b.construction_year.map(|y| y.to_string()).unwrap_or_else(|| "-".into())}</td>
                                <td>
                                    <Link<Route> to={Route::BuildingApartments { id: b.id }} classes="btn btn-sm btn-secondary">
                                        {"View Apartments"}
                                    </Link<Route>>
                                </td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
