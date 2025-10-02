use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct HealthResponse {
    status: String,
    message: String,
}

#[function_component(Home)]
pub fn home() -> Html {
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
