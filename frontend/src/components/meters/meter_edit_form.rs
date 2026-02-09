use crate::i18n::{t, t_with_args};
use crate::services::{ApiError, api_client};
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct Meter {
    pub id: u64,
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    pub is_visible_to_renters: bool,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
    pub last_calibration_date: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Serialize)]
struct UpdateMeterRequest {
    meter_type: Option<String>,
    serial_number: Option<String>,
    is_visible_to_renters: Option<bool>,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct MeterEditFormProps {
    pub meter: Meter,
    pub token: Option<String>,
    pub on_success: Callback<Meter>,
    pub on_error: Callback<String>,
    pub on_cancel: Callback<()>,
}

/// Form for editing meter details or replacing meter (e.g., after calibration)
/// Only displayed for Admin/Manager roles
#[function_component(MeterEditForm)]
pub fn meter_edit_form(props: &MeterEditFormProps) -> Html {
    let edit_serial = use_state(|| props.meter.serial_number.clone());
    let edit_meter_type = use_state(|| props.meter.meter_type.clone());
    let edit_installation_date =
        use_state(|| props.meter.installation_date.clone().unwrap_or_default());
    let edit_calibration_due =
        use_state(|| props.meter.calibration_due_date.clone().unwrap_or_default());
    let edit_visible = use_state(|| props.meter.is_visible_to_renters);
    let updating = use_state(|| false);

    let on_serial_change = {
        let edit_serial = edit_serial.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_serial.set(input.value());
        })
    };

    let on_meter_type_change = {
        let edit_meter_type = edit_meter_type.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            edit_meter_type.set(select.value());
        })
    };

    let on_installation_change = {
        let edit_installation_date = edit_installation_date.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_installation_date.set(input.value());
        })
    };

    let on_calibration_change = {
        let edit_calibration_due = edit_calibration_due.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_calibration_due.set(input.value());
        })
    };

    let on_visible_change = {
        let edit_visible = edit_visible.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            edit_visible.set(input.checked());
        })
    };

    let on_submit = {
        let edit_serial = edit_serial.clone();
        let edit_meter_type = edit_meter_type.clone();
        let edit_installation_date = edit_installation_date.clone();
        let edit_calibration_due = edit_calibration_due.clone();
        let edit_visible = edit_visible.clone();
        let updating = updating.clone();
        let on_success = props.on_success.clone();
        let on_error = props.on_error.clone();
        let token = props.token.clone();
        let meter_id = props.meter.id;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if edit_serial.trim().is_empty() {
                on_error.emit(t("meters-serial-required"));
                return;
            }

            updating.set(true);

            let payload = UpdateMeterRequest {
                meter_type: Some((*edit_meter_type).clone()),
                serial_number: Some((*edit_serial).clone()),
                is_visible_to_renters: Some(*edit_visible),
                installation_date: if edit_installation_date.is_empty() {
                    None
                } else {
                    Some((*edit_installation_date).clone())
                },
                calibration_due_date: if edit_calibration_due.is_empty() {
                    None
                } else {
                    Some((*edit_calibration_due).clone())
                },
            };

            let token = token.clone();
            let on_success = on_success.clone();
            let on_error = on_error.clone();
            let updating = updating.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .put::<_, Meter>(&format!("/meters/{}", meter_id), &payload)
                    .await
                {
                    Ok(updated_meter) => {
                        on_success.emit(updated_meter);
                        updating.set(false);
                    }
                    Err(ApiError::Forbidden) => {
                        on_error.emit(t("meters-permission-denied"));
                        updating.set(false);
                    }
                    Err(e) => {
                        on_error.emit(t_with_args(
                            "meters-failed-load",
                            &[("error", &e.to_string())],
                        ));
                        updating.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="card mb-4">
            <div class="card-header">
                <h5>{t("meters-edit-title")}</h5>
                <p class="mb-0 text-muted small">{t("meters-edit-desc")}</p>
            </div>
            <div class="card-body">
                <form onsubmit={on_submit}>
                    <div class="row">
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-detail-meter-type")}</label>
                            <select class="form-select" value={(*edit_meter_type).clone()} onchange={on_meter_type_change} required=true>
                                <option value="ColdWater">{t("meters-cold-water")}</option>
                                <option value="HotWater">{t("meters-hot-water")}</option>
                                <option value="Gas">{t("meters-gas")}</option>
                                <option value="Electricity">{t("meters-electricity")}</option>
                            </select>
                        </div>
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-serial-number")}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={(*edit_serial).clone()}
                                oninput={on_serial_change}
                                placeholder={t("meters-edit-serial-placeholder")}
                                required=true
                            />
                            <div class="form-text">{t("meters-edit-serial-help")}</div>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-installation-date")}</label>
                            <input
                                type="date"
                                class="form-control"
                                value={(*edit_installation_date).clone()}
                                oninput={on_installation_change}
                            />
                        </div>
                        <div class="col-md-6 mb-3">
                            <label class="form-label">{t("meters-calibration-due")}</label>
                            <input
                                type="date"
                                class="form-control"
                                value={(*edit_calibration_due).clone()}
                                oninput={on_calibration_change}
                            />
                            <div class="form-text">{t("meters-edit-calibration-help")}</div>
                        </div>
                    </div>
                    <div class="mb-3">
                        <div class="form-check">
                            <input
                                class="form-check-input"
                                type="checkbox"
                                id="edit-visible-check"
                                checked={*edit_visible}
                                onchange={on_visible_change}
                            />
                            <label class="form-check-label" for="edit-visible-check">
                                {t("meters-visible-to-renters")}
                            </label>
                        </div>
                    </div>
                    <div class="d-flex gap-2">
                        <button type="submit" class="btn btn-primary" disabled={*updating}>
                            if *updating {
                                <span class="spinner-border spinner-border-sm me-2"></span>
                            }
                            {t("meters-update-meter")}
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
