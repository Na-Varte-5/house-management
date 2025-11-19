use yew::prelude::*;
use serde::Deserialize;
use crate::utils::api::api_url;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct HealthResponse { status: String, message: String }

#[function_component(HealthPage)]
pub fn health_page() -> Html {
    let state = use_state(|| None::<HealthResponse>);
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = reqwasm::http::Request::get(&api_url("/api/v1/health")).send().await {
                    if let Ok(json) = resp.json::<HealthResponse>().await { state.set(Some(json)); }
                }
            });
            || ()
        });
    }
    html!{
        <div class="container mt-4">
            <h1>{"System Health"}</h1>
            { match &*state { Some(h) => html!{<p class="text-success">{format!("{} - {}", h.status, h.message)}</p>} , None => html!{<p class="text-muted">{"Loading..."}</p>} } }
        </div>
    }
}

