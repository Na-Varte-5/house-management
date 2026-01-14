use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::{api_client, ApiError};

#[derive(Serialize)]
struct CreateMeterRequest {
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    is_visible_to_renters: bool,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
}

#[derive(Deserialize)]
struct Meter {
    id: u64,
    apartment_id: u64,
}

#[derive(Deserialize, Clone)]
struct Building {
    id: u64,
    address: String,
}

#[derive(Deserialize, Clone)]
struct Apartment {
    id: u64,
    building_id: u64,
    number: String,
}

#[function_component(MeterNewPage)]
pub fn meter_new_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    // Access control
    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    {"Access denied. Only Admins and Managers can register meters."}
                </div>
            </div>
        };
    }

    // Form state
    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| 0u64);
    let apartment_id = use_state(|| 0u64);
    let meter_type = use_state(|| "ColdWater".to_string());
    let serial_number = use_state(String::default);
    let is_visible_to_renters = use_state(|| true);
    let installation_date = use_state(String::default);
    let calibration_due_date = use_state(String::default);

    let loading_buildings = use_state(|| true);
    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let token = auth.token().map(|t| t.to_string());

    // Load buildings on mount
    {
        let buildings = buildings.clone();
        let loading_buildings = loading_buildings.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings").await {
                    Ok(list) => {
                        buildings.set(list);
                        loading_buildings.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load buildings: {}", e)));
                        loading_buildings.set(false);
                    }
                }
            });
            || ()
        });
    }

    // Load apartments when building changes
    {
        let apartments = apartments.clone();
        let apartment_id = apartment_id.clone();
        let error = error.clone();
        let token = token.clone();
        let building_id = *selected_building;

        use_effect_with(building_id, move |id| {
            if *id != 0 {
                let id = *id;
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", id)).await {
                        Ok(list) => {
                            apartments.set(list.clone());
                            // Auto-select first apartment
                            if let Some(first) = list.first() {
                                apartment_id.set(first.id);
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load apartments: {}", e)));
                        }
                    }
                });
            } else {
                apartments.set(Vec::new());
            }
            || ()
        });
    }

    let on_building_change = {
        let selected_building = selected_building.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            selected_building.set(input.value().parse().unwrap_or(0));
        })
    };

    let on_apartment_change = {
        let apartment_id = apartment_id.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            apartment_id.set(input.value().parse().unwrap_or(0));
        })
    };

    let on_meter_type_change = {
        let meter_type = meter_type.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            meter_type.set(input.value());
        })
    };

    let on_serial_change = {
        let serial_number = serial_number.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            serial_number.set(input.value());
        })
    };

    let on_visible_change = {
        let is_visible_to_renters = is_visible_to_renters.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            is_visible_to_renters.set(input.checked());
        })
    };

    let on_installation_date_change = {
        let installation_date = installation_date.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            installation_date.set(input.value());
        })
    };

    let on_calibration_due_change = {
        let calibration_due_date = calibration_due_date.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            calibration_due_date.set(input.value());
        })
    };

    let on_submit = {
        let apartment_id = apartment_id.clone();
        let meter_type = meter_type.clone();
        let serial_number = serial_number.clone();
        let is_visible_to_renters = is_visible_to_renters.clone();
        let installation_date = installation_date.clone();
        let calibration_due_date = calibration_due_date.clone();
        let submitting = submitting.clone();
        let success = success.clone();
        let error = error.clone();
        let navigator = navigator.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if *apartment_id == 0 {
                error.set(Some("Please select an apartment".to_string()));
                return;
            }

            if serial_number.trim().is_empty() {
                error.set(Some("Serial number is required".to_string()));
                return;
            }

            submitting.set(true);
            error.set(None);

            let payload = CreateMeterRequest {
                apartment_id: *apartment_id,
                meter_type: (*meter_type).clone(),
                serial_number: (*serial_number).clone(),
                is_visible_to_renters: *is_visible_to_renters,
                installation_date: if installation_date.is_empty() { None } else { Some((*installation_date).clone()) },
                calibration_due_date: if calibration_due_date.is_empty() { None } else { Some((*calibration_due_date).clone()) },
            };

            let token = token.clone();
            let success = success.clone();
            let error = error.clone();
            let submitting = submitting.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post::<_, Meter>("/meters", &payload).await {
                    Ok(meter) => {
                        success.set(Some("Meter registered successfully".to_string()));
                        submitting.set(false);

                        // Navigate to meter detail after short delay
                        gloo_timers::callback::Timeout::new(1500, move || {
                            navigator.push(&Route::ApartmentMeters { apartment_id: meter.apartment_id });
                        }).forget();
                    }
                    Err(ApiError::Forbidden) => {
                        error.set(Some("Permission denied".to_string()));
                        submitting.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to register meter: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_cancel = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Buildings))
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
            <h2>{"Register New Meter"}</h2>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if *loading_buildings {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else {
                <div class="card">
                    <div class="card-body">
                        <form onsubmit={on_submit}>
                            <div class="mb-3">
                                <label class="form-label">{"Building"}</label>
                                <select class="form-select" value={selected_building.to_string()} onchange={on_building_change} required=true>
                                    <option value="0">{"-- Select Building --"}</option>
                                    { for buildings.iter().map(|b| html! {
                                        <option key={b.id} value={b.id.to_string()}>{&b.address}</option>
                                    }) }
                                </select>
                            </div>

                            <div class="mb-3">
                                <label class="form-label">{"Apartment"}</label>
                                <select class="form-select" value={apartment_id.to_string()} onchange={on_apartment_change} required=true disabled={apartments.is_empty()}>
                                    <option value="0">{"-- Select Apartment --"}</option>
                                    { for apartments.iter().map(|a| html! {
                                        <option key={a.id} value={a.id.to_string()}>{"Apartment "}{&a.number}</option>
                                    }) }
                                </select>
                            </div>

                            <div class="mb-3">
                                <label class="form-label">{"Meter Type"}</label>
                                <select class="form-select" value={(*meter_type).clone()} onchange={on_meter_type_change}>
                                    <option value="ColdWater">{"Cold Water"}</option>
                                    <option value="HotWater">{"Hot Water"}</option>
                                    <option value="Gas">{"Gas"}</option>
                                    <option value="Electricity">{"Electricity"}</option>
                                </select>
                            </div>

                            <div class="mb-3">
                                <label class="form-label">{"Serial Number"}</label>
                                <input
                                    type="text"
                                    class="form-control"
                                    value={(*serial_number).clone()}
                                    oninput={on_serial_change}
                                    placeholder="Enter meter serial number"
                                    required=true
                                />
                            </div>

                            <div class="mb-3">
                                <label class="form-label">{"Installation Date"}</label>
                                <input
                                    type="date"
                                    class="form-control"
                                    value={(*installation_date).clone()}
                                    oninput={on_installation_date_change}
                                />
                            </div>

                            <div class="mb-3">
                                <label class="form-label">{"Calibration Due Date"}</label>
                                <input
                                    type="date"
                                    class="form-control"
                                    value={(*calibration_due_date).clone()}
                                    oninput={on_calibration_due_change}
                                />
                                <div class="form-text">{"When the meter needs to be recalibrated/renewed"}</div>
                            </div>

                            <div class="mb-3 form-check">
                                <input
                                    type="checkbox"
                                    class="form-check-input"
                                    id="visibleToRenters"
                                    checked={*is_visible_to_renters}
                                    onchange={on_visible_change}
                                />
                                <label class="form-check-label" for="visibleToRenters">
                                    {"Visible to Renters"}
                                </label>
                            </div>

                            <div class="d-flex gap-2">
                                <button type="submit" class="btn btn-primary" disabled={*submitting}>
                                    if *submitting {
                                        <span class="spinner-border spinner-border-sm me-2"></span>
                                    }
                                    {"Register Meter"}
                                </button>
                                <button type="button" class="btn btn-secondary" onclick={on_cancel}>
                                    {"Cancel"}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            }
        </div>
    }
}
