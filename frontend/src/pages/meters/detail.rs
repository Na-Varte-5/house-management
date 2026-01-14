use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Deserialize, Clone, PartialEq)]
struct Meter {
    id: u64,
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    is_visible_to_renters: bool,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
    last_calibration_date: Option<String>,
    is_active: bool,
    created_at: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct MeterReading {
    id: u64,
    meter_id: u64,
    reading_value: String,
    reading_timestamp: String,
    unit: String,
    source: String,
    created_at: Option<String>,
}

#[derive(Serialize)]
struct CreateReadingRequest {
    reading_value: String,
    timestamp: Option<String>,
    unit: String,
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
pub struct Props {
    pub id: u64,
}

#[function_component(MeterDetailPage)]
pub fn meter_detail_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let meter = use_state(|| None::<Meter>);
    let readings = use_state(|| Vec::<MeterReading>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Manual entry form state
    let show_entry_form = use_state(|| false);
    let entry_value = use_state(String::default);
    let entry_unit = use_state(|| "m3".to_string());
    let submitting = use_state(|| false);

    // Edit/Replace meter form state
    let show_edit_form = use_state(|| false);
    let edit_serial = use_state(String::default);
    let edit_meter_type = use_state(String::default);
    let edit_installation_date = use_state(String::default);
    let edit_calibration_due = use_state(String::default);
    let edit_visible = use_state(|| true);
    let updating = use_state(|| false);

    let meter_id = props.id;
    let token = auth.token().map(|t| t.to_string());
    let is_admin_or_manager = auth.is_admin_or_manager();

    // Load meter details and readings
    {
        let meter = meter.clone();
        let readings = readings.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(meter_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());

                // Load meter details
                match client.get::<Meter>(&format!("/meters/{}", id)).await {
                    Ok(m) => {
                        meter.set(Some(m));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load meter: {}", e)));
                        loading.set(false);
                        return;
                    }
                }

                // Load readings
                match client.get::<Vec<MeterReading>>(&format!("/meters/{}/readings", id)).await {
                    Ok(list) => {
                        readings.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load readings: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_back = {
        let navigator = navigator.clone();
        let meter = meter.clone();
        Callback::from(move |_| {
            if let Some(ref m) = *meter {
                navigator.push(&Route::ApartmentMeters { apartment_id: m.apartment_id });
            } else {
                navigator.push(&Route::Buildings);
            }
        })
    };

    let toggle_entry_form = {
        let show_entry_form = show_entry_form.clone();
        Callback::from(move |_| show_entry_form.set(!*show_entry_form))
    };

    let toggle_edit_form = {
        let show_edit_form = show_edit_form.clone();
        let meter = meter.clone();
        let edit_serial = edit_serial.clone();
        let edit_meter_type = edit_meter_type.clone();
        let edit_installation_date = edit_installation_date.clone();
        let edit_calibration_due = edit_calibration_due.clone();
        let edit_visible = edit_visible.clone();
        Callback::from(move |_| {
            let is_showing = *show_edit_form;
            if !is_showing {
                // Populate form with current values
                if let Some(ref m) = *meter {
                    edit_serial.set(m.serial_number.clone());
                    edit_meter_type.set(m.meter_type.clone());
                    edit_installation_date.set(m.installation_date.clone().unwrap_or_default());
                    edit_calibration_due.set(m.calibration_due_date.clone().unwrap_or_default());
                    edit_visible.set(m.is_visible_to_renters);
                }
            }
            show_edit_form.set(!is_showing);
        })
    };

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

    let on_edit_serial_change = {
        let edit_serial = edit_serial.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_serial.set(input.value());
        })
    };

    let on_edit_meter_type_change = {
        let edit_meter_type = edit_meter_type.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            edit_meter_type.set(select.value());
        })
    };

    let on_edit_installation_change = {
        let edit_installation_date = edit_installation_date.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_installation_date.set(input.value());
        })
    };

    let on_edit_calibration_change = {
        let edit_calibration_due = edit_calibration_due.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            edit_calibration_due.set(input.value());
        })
    };

    let on_edit_visible_change = {
        let edit_visible = edit_visible.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            edit_visible.set(input.checked());
        })
    };

    let on_submit_update = {
        let edit_serial = edit_serial.clone();
        let edit_meter_type = edit_meter_type.clone();
        let edit_installation_date = edit_installation_date.clone();
        let edit_calibration_due = edit_calibration_due.clone();
        let edit_visible = edit_visible.clone();
        let updating = updating.clone();
        let success = success.clone();
        let error = error.clone();
        let show_edit_form = show_edit_form.clone();
        let meter = meter.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if edit_serial.trim().is_empty() {
                error.set(Some("Serial number is required".to_string()));
                return;
            }

            updating.set(true);
            error.set(None);

            let payload = UpdateMeterRequest {
                meter_type: Some((*edit_meter_type).clone()),
                serial_number: Some((*edit_serial).clone()),
                is_visible_to_renters: Some(*edit_visible),
                installation_date: if edit_installation_date.is_empty() { None } else { Some((*edit_installation_date).clone()) },
                calibration_due_date: if edit_calibration_due.is_empty() { None } else { Some((*edit_calibration_due).clone()) },
            };

            let token = token.clone();
            let success = success.clone();
            let error = error.clone();
            let updating = updating.clone();
            let show_edit_form = show_edit_form.clone();
            let meter = meter.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.put::<_, Meter>(&format!("/meters/{}", meter_id), &payload).await {
                    Ok(updated_meter) => {
                        success.set(Some("Meter updated successfully".to_string()));
                        updating.set(false);
                        show_edit_form.set(false);
                        meter.set(Some(updated_meter));
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("Permission denied".to_string()));
                        updating.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to update meter: {}", e)));
                        updating.set(false);
                    }
                }
            });
        })
    };

    let on_submit_reading = {
        let entry_value = entry_value.clone();
        let entry_unit = entry_unit.clone();
        let submitting = submitting.clone();
        let success = success.clone();
        let error = error.clone();
        let show_entry_form = show_entry_form.clone();
        let readings = readings.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if entry_value.trim().is_empty() {
                error.set(Some("Reading value is required".to_string()));
                return;
            }

            submitting.set(true);
            error.set(None);

            let payload = CreateReadingRequest {
                reading_value: (*entry_value).clone(),
                timestamp: None, // Use current time
                unit: (*entry_unit).clone(),
            };

            let token = token.clone();
            let success = success.clone();
            let error = error.clone();
            let submitting = submitting.clone();
            let show_entry_form = show_entry_form.clone();
            let entry_value = entry_value.clone();
            let readings = readings.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post::<_, MeterReading>(&format!("/meters/{}/readings", meter_id), &payload).await {
                    Ok(new_reading) => {
                        success.set(Some("Reading recorded successfully".to_string()));
                        submitting.set(false);
                        show_entry_form.set(false);
                        entry_value.set(String::default());

                        // Add new reading to the list
                        let mut updated_readings = (*readings).clone();
                        updated_readings.insert(0, new_reading);
                        readings.set(updated_readings);
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("Permission denied".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to record reading: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_export_csv = {
        let token = token.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let token = token.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<String>(&format!("/meters/{}/readings/export", meter_id)).await {
                    Ok(_csv) => {
                        // Open in new tab
                        if let Some(window) = web_sys::window() {
                            let _ = window.open_with_url(&format!("/api/v1/meters/{}/readings/export", meter_id));
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to export: {}", e)));
                    }
                }
            });
        })
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    html! {
        <div class="container mt-4">
            <div class="d-flex justify-content-between align-items-center mb-3">
                <div>
                    <button class="btn btn-outline-secondary me-2" onclick={on_back}>
                        <i class="bi bi-arrow-left"></i> {"Back"}
                    </button>
                    <h2 class="d-inline">{"Meter Details"}</h2>
                </div>
                <div>
                    if is_admin_or_manager {
                        <>
                            <button class="btn btn-success me-2" onclick={toggle_entry_form.clone()}>
                                {"+ Add Reading"}
                            </button>
                            <button class="btn btn-warning me-2" onclick={toggle_edit_form.clone()}>
                                <i class="bi bi-pencil"></i> {"Edit/Replace Meter"}
                            </button>
                        </>
                    }
                    <button class="btn btn-outline-primary" onclick={on_export_csv}>
                        <i class="bi bi-download"></i> {"Export CSV"}
                    </button>
                </div>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if let Some(ref m) = *meter {
                <div>
                    // Meter info card
                    <div class="card mb-4">
                        <div class="card-body">
                            <div class="row">
                                <div class="col-md-6">
                                    <p><strong>{"Type:"}</strong> {&m.meter_type}</p>
                                    <p><strong>{"Serial Number:"}</strong> {&m.serial_number}</p>
                                    if let Some(ref inst_date) = m.installation_date {
                                        <p><strong>{"Installation Date:"}</strong> {inst_date}</p>
                                    }
                                </div>
                                <div class="col-md-6">
                                    if let Some(ref cal_date) = m.calibration_due_date {
                                        <p><strong>{"Calibration Due:"}</strong> {cal_date}</p>
                                    }
                                    if let Some(ref last_cal) = m.last_calibration_date {
                                        <p><strong>{"Last Calibration:"}</strong> {last_cal}</p>
                                    }
                                    <p>
                                        <strong>{"Visible to Renters:"}</strong>
                                        {" "}
                                        if m.is_visible_to_renters {
                                            <span class="badge bg-success">{"Yes"}</span>
                                        } else {
                                            <span class="badge bg-secondary">{"No"}</span>
                                        }
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Manual entry form
                    if *show_entry_form && is_admin_or_manager {
                        <div class="card mb-4">
                            <div class="card-header">
                                <h5>{"Add Manual Reading"}</h5>
                            </div>
                            <div class="card-body">
                                <form onsubmit={on_submit_reading}>
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
                                        <button type="button" class="btn btn-secondary" onclick={toggle_entry_form}>
                                            {"Cancel"}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    }

                    // Edit/Replace meter form
                    if *show_edit_form && is_admin_or_manager {
                        <div class="card mb-4">
                            <div class="card-header">
                                <h5>{"Edit/Replace Meter"}</h5>
                                <p class="mb-0 text-muted small">{"Update meter details or replace with a new serial number (e.g., after calibration)"}</p>
                            </div>
                            <div class="card-body">
                                <form onsubmit={on_submit_update}>
                                    <div class="row">
                                        <div class="col-md-6 mb-3">
                                            <label class="form-label">{"Meter Type"}</label>
                                            <select class="form-select" value={(*edit_meter_type).clone()} onchange={on_edit_meter_type_change} required=true>
                                                <option value="ColdWater">{"Cold Water"}</option>
                                                <option value="HotWater">{"Hot Water"}</option>
                                                <option value="Gas">{"Gas"}</option>
                                                <option value="Electricity">{"Electricity"}</option>
                                            </select>
                                        </div>
                                        <div class="col-md-6 mb-3">
                                            <label class="form-label">{"Serial Number"}</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                value={(*edit_serial).clone()}
                                                oninput={on_edit_serial_change}
                                                placeholder="Enter new serial number"
                                                required=true
                                            />
                                            <div class="form-text">{"Update this when replacing the physical meter"}</div>
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col-md-6 mb-3">
                                            <label class="form-label">{"Installation Date"}</label>
                                            <input
                                                type="date"
                                                class="form-control"
                                                value={(*edit_installation_date).clone()}
                                                oninput={on_edit_installation_change}
                                            />
                                        </div>
                                        <div class="col-md-6 mb-3">
                                            <label class="form-label">{"Calibration Due Date"}</label>
                                            <input
                                                type="date"
                                                class="form-control"
                                                value={(*edit_calibration_due).clone()}
                                                oninput={on_edit_calibration_change}
                                            />
                                            <div class="form-text">{"Set new calibration due date after replacement"}</div>
                                        </div>
                                    </div>
                                    <div class="mb-3">
                                        <div class="form-check">
                                            <input
                                                class="form-check-input"
                                                type="checkbox"
                                                id="edit-visible-check"
                                                checked={*edit_visible}
                                                onchange={on_edit_visible_change}
                                            />
                                            <label class="form-check-label" for="edit-visible-check">
                                                {"Visible to Renters"}
                                            </label>
                                        </div>
                                    </div>
                                    <div class="d-flex gap-2">
                                        <button type="submit" class="btn btn-primary" disabled={*updating}>
                                            if *updating {
                                                <span class="spinner-border spinner-border-sm me-2"></span>
                                            }
                                            {"Update Meter"}
                                        </button>
                                        <button type="button" class="btn btn-secondary" onclick={toggle_edit_form}>
                                            {"Cancel"}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    }

                    // Readings table
                    <div class="card">
                        <div class="card-header">
                            <h5>{"Reading History"}</h5>
                        </div>
                        <div class="card-body">
                            if readings.is_empty() {
                                <div class="alert alert-info">
                                    {"No readings recorded yet."}
                                </div>
                            } else {
                                <div class="table-responsive">
                                    <table class="table table-striped">
                                        <thead>
                                            <tr>
                                                <th>{"Timestamp"}</th>
                                                <th>{"Value"}</th>
                                                <th>{"Unit"}</th>
                                                <th>{"Source"}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            { for readings.iter().map(|reading| html! {
                                                <tr key={reading.id}>
                                                    <td>{&reading.reading_timestamp}</td>
                                                    <td>{&reading.reading_value}</td>
                                                    <td>{&reading.unit}</td>
                                                    <td>
                                                        <span class={if reading.source == "Webhook" { "badge bg-info" } else { "badge bg-secondary" }}>
                                                            {&reading.source}
                                                        </span>
                                                    </td>
                                                </tr>
                                            }) }
                                        </tbody>
                                    </table>
                                </div>
                            }
                        </div>
                    </div>
                </div>
            } else {
                <div class="alert alert-danger">
                    {"Meter not found"}
                </div>
            }
        </div>
    }
}
