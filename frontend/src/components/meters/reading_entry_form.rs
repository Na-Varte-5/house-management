use crate::i18n::{t, t_with_args};
use crate::services::{ApiError, api_client};
use serde::Serialize;
use web_sys::HtmlInputElement;
use yew::prelude::*;

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
                on_error.emit(t("meters-reading-required"));
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
                match client
                    .post::<_, serde_json::Value>(
                        &format!("/meters/{}/readings", meter_id),
                        &payload,
                    )
                    .await
                {
                    Ok(_) => {
                        on_success.emit(t("meters-reading-success"));
                        submitting.set(false);
                        entry_value.set(String::default());
                    }
                    Err(ApiError::Forbidden) => {
                        on_error.emit(t("meters-permission-denied"));
                        submitting.set(false);
                    }
                    Err(e) => {
                        on_error.emit(t_with_args(
                            "meters-failed-load",
                            &[("error", &e.to_string())],
                        ));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="card mb-4">
            <div class="card-header">
                <h5>{t("meters-add-reading")}</h5>
            </div>
            <div class="card-body">
                <form onsubmit={on_submit}>
                    <div class="row">
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-reading-value-label")}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={(*entry_value).clone()}
                                oninput={on_value_change}
                                placeholder={t("meters-reading-value-placeholder")}
                                required=true
                            />
                        </div>
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-reading-unit-label")}</label>
                            <select class="form-select" value={(*entry_unit).clone()} onchange={on_unit_change}>
                                <option value="m3">{t("meters-unit-m3")}</option>
                                <option value="kWh">{t("meters-unit-kwh")}</option>
                                <option value="L">{t("meters-unit-liters")}</option>
                            </select>
                        </div>
                    </div>
                    <div class="d-flex gap-2">
                        <button type="submit" class="btn btn-primary" disabled={*submitting}>
                            if *submitting {
                                <span class="spinner-border spinner-border-sm me-2"></span>
                            }
                            {t("meters-save-reading")}
                        </button>
                        <button type="button" class="btn btn-secondary" onclick={props.on_cancel.reform(|_| ())}>
                            {t("button-cancel")}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
