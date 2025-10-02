use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;

#[derive(Routable, PartialEq, Clone, Debug)]
enum Route {
    #[at("/")]
    Home,
    #[at("/buildings")]
    Buildings,
    #[at("/buildings/:id/apartments")]
    BuildingApartments { id: u64 },
}

#[function_component(Navbar)]
fn navbar() -> Html {
    html! {
        <nav class="navbar navbar-expand navbar-dark bg-dark">
            <div class="container-fluid">
                <Link<Route> to={Route::Home} classes="navbar-brand">{"HouseMgmt"}</Link<Route>>
                <div class="navbar-nav">
                    <Link<Route> to={Route::Buildings} classes="nav-link">{"Buildings"}</Link<Route>>
                </div>
            </div>
        </nav>
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct HealthResponse { status: String, message: String }

#[function_component(Home)]
fn home() -> Html {
    let state = use_state(|| None::<HealthResponse>);
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match reqwasm::http::Request::get("/api/v1/health").send().await {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<HealthResponse>().await {
                            state.set(Some(json));
                        }
                    }
                    Err(_) => {}
                }
            });
            || ()
        });
    }

    html! {
        <div class="container mt-4">
            <h1>{"Dashboard"}</h1>
            if let Some(health) = (*state).clone() {
                <p class="text-success">{format!("{}: {}", health.status, health.message)}</p>
            } else {
                <p>{"Loading health..."}</p>
            }
        </div>
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Building { id: u64, address: String, construction_year: Option<i32> }

#[function_component(BuildingsPage)]
fn buildings_page() -> Html {
    let buildings = use_state(|| Vec::<Building>::new());
    let address = use_state(String::default);
    let year = use_state(|| String::new());

    {
        let buildings = buildings.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = reqwasm::http::Request::get("/api/v1/buildings").send().await {
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
                if let Ok(resp) = reqwasm::http::Request::post("/api/v1/buildings")
                    .header("Content-Type", "application/json")
                    .body(payload.to_string())
                    .send()
                    .await
                {
                    if resp.ok() {
                        // Reload list
                        if let Ok(resp2) = reqwasm::http::Request::get("/api/v1/buildings").send().await {
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

    html! {
        <div class="container mt-4">
            <h2>{"Buildings"}</h2>
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Apartment { id: u64, building_id: u64, number: String, size_sq_m: Option<f64> }

#[derive(Properties, PartialEq, Clone)]
struct ApartmentsProps { id: u64 }

#[function_component(BuildingApartmentsPage)]
fn building_apartments_page() -> Html {
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
                if let Ok(resp) = reqwasm::http::Request::get(&url).send().await {
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
                if let Ok(resp) = reqwasm::http::Request::post("/api/v1/apartments")
                    .header("Content-Type", "application/json")
                    .body(payload.to_string())
                    .send().await {
                    if resp.ok() {
                        let url = format!("/api/v1/buildings/{}/apartments", id2);
                        if let Ok(resp2) = reqwasm::http::Request::get(&url).send().await {
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

    html! {
        <div class="container mt-4">
            <h3>{format!("Apartments in building {}", id)}</h3>
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

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" />
            <Navbar />
            <Switch<Route> render={move |route| match route {
                Route::Home => html!{<Home />},
                Route::Buildings => html!{<BuildingsPage />},
                Route::BuildingApartments { .. } => html!{<BuildingApartmentsPage />},
            }} />
        </BrowserRouter>
    }
}
