use yew::prelude::*;
use serde::Deserialize;
use crate::services::api_client;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct HealthResponse {
    status: String,
    message: String,
}

#[function_component(HealthPage)]
pub fn health_page() -> Html {
    let state = use_state(|| None::<HealthResponse>);
    let error = use_state(|| None::<String>);

    {
        let state = state.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(None);
                match client.get::<HealthResponse>("/health").await {
                    Ok(health) => state.set(Some(health)),
                    Err(e) => error.set(Some(format!("Health check failed: {}", e))),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container mt-4">
            <h1>{"System Health"}</h1>
            {
                if let Some(err) = (*error).clone() {
                    html! { <p class="text-danger">{err}</p> }
                } else if let Some(h) = (*state).clone() {
                    html! {
                        <div class="alert alert-success">
                            <strong>{"Status: "}</strong>{&h.status}
                            <br />
                            <strong>{"Message: "}</strong>{&h.message}
                        </div>
                    }
                } else {
                    html! {
                        <div class="text-center py-5">
                            <div class="spinner-border" role="status">
                                <span class="visually-hidden">{"Loading..."}</span>
                            </div>
                        </div>
                    }
                }
            }
        </div>
    }
}
