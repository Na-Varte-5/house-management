use yew::prelude::*;
use serde::Serialize;
use web_sys::HtmlInputElement;
use crate::services::{api_client, ApiError};

#[derive(Serialize)]
struct CreateReadingRequest {
    reading_value: String,
    timestamp: Option<String>,
    unit: String,
}

#[derive(Properties, PartialEq)]
pub struct ReadingEntryFormProps {
    pub meter_id: u64,
    pub token: Option<String>,
    pub on_success: Callback<String>,
    pub on_error: Callback<String>,
    pub on_cancel: Callback<()>,
}

/// Form for manually entering meter readings
/// Only displayed for Admin/Manager roles
#[function_component(ReadingEntryForm)]
pub fn reading_entry_form(props: &ReadingEntryFormProps) -> Html {
    let entry_value = use_state(String::default);
    let entry_unit = use_state(|| "m3".to_string());
    let submitting = use_state(|| false);

    let on_value_change = {
        let entry_value = entry_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            entry_value.set(input.value());
        })
    };

    let on_unit_change = {
        let entry_unit = entry_unit.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            entry_unit.set(input.value());
        })
    };

    let on_submit = {
        let entry_value = entry_value.clone();
        let entry_unit = entry_unit.clone();
        let submitting = submitting.clone();
        let on_success = props.on_success.clone();
        let on_error = props.on_error.clone();
        let token = props.token.clone();
        let meter_id = props.meter_id;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if entry_value.trim().is_empty() {
                on_error.emit("Reading value is required".to_string());
                return;
            }

            submitting.set(true);

            let payload = CreateReadingRequest {
                reading_value: (*entry_value).clone(),
                timestamp: None, // Use current time
                unit: (*entry_unit).clone(),
            };

            let token = token.clone();
            let on_success = on_success.clone();
            let on_error = on_error.clone();
            let submitting = submitting.clone();
            let entry_value = entry_value.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post::<_, serde_json::Value>(&format!("/meters/{}/readings", meter_id), &payload).await {
                    Ok(_) => {
                        on_success.emit("Reading recorded successfully".to_string());
                        submitting.set(false);
                        entry_value.set(String::default());
                    }
                    Err(ApiError::Forbidden) => {
                        on_error.emit("Permission denied".to_string());
                        submitting.set(false);
                    }
                    Err(e) => {
                        on_error.emit(format!("Failed to record reading: {}", e));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="card mb-4">
            <div class="card-header">
                <h5>{"Add Manual Reading"}</h5>
            </div>
            <div class="card-body">
                <form onsubmit={on_submit}>
                    <div class="row">
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{"Reading Value"}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={(*entry_value).clone()}
                                oninput={on_value_change}
                                placeholder="123.456"
                                required=true
                            />
                        </div>
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{"Unit"}</label>
                            <select class="form-select" value={(*entry_unit).clone()} onchange={on_unit_change}>
                                <option value="m3">{"m³ (cubic meters)"}</option>
                                <option value="kWh">{"kWh (kilowatt-hours)"}</option>
                                <option value="L">{"L (liters)"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="d-flex gap-2">
                        <button type="submit" class="btn btn-primary" disabled={*submitting}>
                            if *submitting {
                                <span class="spinner-border spinner-border-sm me-2"></span>
                            }
                            {"Save Reading"}
                        </button>
                        <button type="button" class="btn btn-secondary" onclick={props.on_cancel.reform(|_| ())}>
                            {"Cancel"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
