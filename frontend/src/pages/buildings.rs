use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;
use crate::utils::api::api_url;
use crate::utils::auth::{get_token, current_user};

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Building {
    id: u64,
    address: String,
    construction_year: Option<i32>,
}

#[function_component(BuildingsPage)]
pub fn buildings_page() -> Html {
    let buildings = use_state(|| Vec::<Building>::new());
    let address = use_state(String::default);
    let year = use_state(|| String::new());

    {
        let buildings = buildings.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = reqwasm::http::Request::get(&api_url("/api/v1/buildings"))
                    .send()
                    .await
                {
                    if let Ok(list) = resp.json::<Vec<Building>>().await {
                        buildings.set(list);
                    }
                }
            });
            || ()
        });
    }

    let on_submit = {
        let address = address.clone();
        let year = year.clone();
        let buildings = buildings.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let payload = serde_json::json!({
                "address": (*address).clone(),
                "construction_year": year.parse::<i32>().ok(),
            });
            let buildings = buildings.clone();
            let address = address.clone();
            let year = year.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url("/api/v1/buildings"))
                    .header("Content-Type", "application/json");
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(resp) = req.body(payload.to_string()).send().await {
                    if resp.ok() {
                        // Reload list
                        if let Ok(resp2) =
                            reqwasm::http::Request::get(&api_url("/api/v1/buildings"))
                                .send()
                                .await
                        {
                            if let Ok(list) = resp2.json::<Vec<Building>>().await {
                                buildings.set(list);
                                address.set(String::new());
                                year.set(String::new());
                            }
                        }
                    }
                }
            });
        })
    };

    let can_create = current_user().map(|u| u.roles.iter().any(|r| r=="Admin" || r=="Manager")).unwrap_or(false);

    html! {
        <div class="container mt-4">
            <h2>{"Buildings"}</h2>
            {
                if can_create {
                    html! {
                        <form class="row g-2" onsubmit={on_submit}>
                            <div class="col-md-6">
                                <input class="form-control" placeholder="Address" value={(*address).clone()} oninput={{
                                    let address = address.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        address.set(input.value());
                                    })
                                }} />
                            </div>
                            <div class="col-md-3">
                                <input class="form-control" placeholder="Year" value={(*year).clone()} oninput={{
                                    let year = year.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        year.set(input.value());
                                    })
                                }} />
                            </div>
                            <div class="col-md-3">
                                <button class="btn btn-primary" type="submit">{"Add"}</button>
                            </div>
                        </form>
                    }
                } else {
                    html! { <div class="alert alert-secondary small">{"You don't have permission to add buildings."}</div> }
                }
            }
            <table class="table table-striped mt-3">
                <thead><tr><th>{"ID"}</th><th>{"Address"}</th><th>{"Year"}</th><th>{"Actions"}</th></tr></thead>
                <tbody>
                    { for (*buildings).iter().map(|b| html!{
                        <tr>
                            <td>{b.id}</td>
                            <td>{b.address.clone()}</td>
                            <td>{b.construction_year.map(|y| y.to_string()).unwrap_or("-".into())}</td>
                            <td>
                                <Link<Route> to={Route::BuildingApartments { id: b.id }} classes="btn btn-sm btn-secondary">{"Apartments"}</Link<Route>>
                            </td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
