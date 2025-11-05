use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;
use crate::utils::api::api_url;
use crate::utils::auth::{get_token, current_user};

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Apartment {
    id: u64,
    building_id: u64,
    number: String,
    size_sq_m: Option<f64>,
}

#[function_component(BuildingApartmentsPage)]
pub fn building_apartments_page() -> Html {
    let route = use_route::<Route>();
    let id = match route {
        Some(Route::BuildingApartments { id }) => id,
        _ => 0,
    };

    let apartments = use_state(|| Vec::<Apartment>::new());
    let number = use_state(String::default);
    let size = use_state(String::default);

    {
        let apartments = apartments.clone();
        let id2 = id;
        use_effect_with(id, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/v1/buildings/{}/apartments", id2);
                if let Ok(resp) = reqwasm::http::Request::get(&api_url(&url)).send().await {
                    if let Ok(list) = resp.json::<Vec<Apartment>>().await {
                        apartments.set(list);
                    }
                }
            });
            || ()
        });
    }

    let on_submit = {
        let number = number.clone();
        let size = size.clone();
        let apartments = apartments.clone();
        let id2 = id;
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let payload = serde_json::json!({
                "building_id": id2,
                "number": (*number).clone(),
                "size_sq_m": size.parse::<f64>().ok(),
            });
            let apartments = apartments.clone();
            let number = number.clone();
            let size = size.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url("/api/v1/apartments"))
                    .header("Content-Type", "application/json");
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(resp) = req.body(payload.to_string()).send().await {
                    if resp.ok() {
                        let url = format!("/api/v1/buildings/{}/apartments", id2);
                        if let Ok(resp2) = reqwasm::http::Request::get(&api_url(&url)).send().await
                        {
                            if let Ok(list) = resp2.json::<Vec<Apartment>>().await {
                                apartments.set(list);
                                number.set(String::new());
                                size.set(String::new());
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
            <h3>{format!("Apartments in building {}", id)}</h3>
            {
                if can_create {
                    html! {
                        <form class="row g-2" onsubmit={on_submit}>
                            <div class="col-md-4">
                                <input class="form-control" placeholder="Number" value={(*number).clone()} oninput={{
                                    let number = number.clone();
                                    Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); number.set(input.value()); })
                                }} />
                            </div>
                            <div class="col-md-4">
                                <input class="form-control" placeholder="Size m2" value={(*size).clone()} oninput={{
                                    let size = size.clone();
                                    Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); size.set(input.value()); })
                                }} />
                            </div>
                            <div class="col-md-4">
                                <button class="btn btn-primary" type="submit">{"Add"}</button>
                            </div>
                        </form>
                    }
                } else {
                    html! { <div class="alert alert-secondary small">{"You don't have permission to add apartments."}</div> }
                }
            }
            <table class="table table-striped mt-3">
                <thead><tr><th>{"ID"}</th><th>{"Number"}</th><th>{"Size"}</th></tr></thead>
                <tbody>
                    { for (*apartments).iter().map(|a| html!{
                        <tr>
                            <td>{a.id}</td>
                            <td>{a.number.clone()}</td>
                            <td>{a.size_sq_m.map(|s| format!("{:.2}", s)).unwrap_or("-".into())}</td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
