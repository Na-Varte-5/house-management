use crate::components::{DateTimeInput, FormGroup, Select, SelectOption, TextInput};
use crate::services::api_client;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct Building {
    pub id: u64,
    pub address: String,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Apartment {
    pub id: u64,
    pub number: String,
    pub building_id: u64,
}

#[derive(Serialize)]
struct NewMeter {
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct MeterRegisterFormProps {
    /// Optional token for API authentication
    pub token: Option<String>,

    /// Callback when meter is successfully registered
    pub on_success: Callback<()>,

    /// Callback to set error message
    pub on_error: Callback<String>,
}

#[function_component(MeterRegisterForm)]
pub fn meter_register_form(props: &MeterRegisterFormProps) -> Html {
    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| "".to_string());
    let selected_apartment = use_state(|| "".to_string());
    let meter_type = use_state(|| "ColdWater".to_string());
    let serial_number = use_state(String::default);
    let installation_date = use_state(String::default);
    let calibration_due = use_state(String::default);
    let submitting = use_state(|| false);

    // Load buildings on mount
    {
        let buildings = buildings.clone();
        let token = props.token.clone();
        let on_error = props.on_error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings").await {
                    Ok(list) => buildings.set(list),
                    Err(e) => on_error.emit(format!("Failed to load buildings: {}", e)),
                }
            });
            || ()
        });
    }

    // Load apartments for selected building
    {
        let apartments = apartments.clone();
        let selected_building = selected_building.clone();
        let token = props.token.clone();

        use_effect_with(selected_building.clone(), move |building_str| {
            if let Ok(building_id) = building_str.parse::<u64>() {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(list) = client
                        .get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", building_id))
                        .await
                    {
                        apartments.set(list);
                    }
                });
            } else {
                apartments.set(Vec::new());
            }
            || ()
        });
    }

    let on_submit = {
        let selected_apartment = selected_apartment.clone();
        let meter_type = meter_type.clone();
        let serial_number = serial_number.clone();
        let installation_date = installation_date.clone();
        let calibration_due = calibration_due.clone();
        let submitting = submitting.clone();
        let on_error = props.on_error.clone();
        let on_success = props.on_success.clone();
        let token = props.token.clone();
        let selected_building = selected_building.clone();
        let apartments_list = apartments.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // Validation
            if selected_apartment.is_empty() {
                on_error.emit("Please select an apartment".to_string());
                return;
            }
            if serial_number.trim().is_empty() {
                on_error.emit("Serial number is required".to_string());
                return;
            }

            let apt_id = match selected_apartment.parse::<u64>() {
                Ok(id) => id,
                Err(_) => {
                    on_error.emit("Invalid apartment selected".to_string());
                    return;
                }
            };

            let meter_type = meter_type.clone();
            let serial_number = serial_number.clone();
            let installation_date = installation_date.clone();
            let calibration_due = calibration_due.clone();
            let submitting = submitting.clone();
            let on_error = on_error.clone();
            let on_success = on_success.clone();
            let token = token.clone();
            let selected_building = selected_building.clone();
            let selected_apartment = selected_apartment.clone();
            let apartments_list = apartments_list.clone();

            submitting.set(true);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());

                let new_meter = NewMeter {
                    apartment_id: apt_id,
                    meter_type: (*meter_type).clone(),
                    serial_number: (*serial_number).clone(),
                    installation_date: if installation_date.is_empty() {
                        None
                    } else {
                        Some((*installation_date).clone())
                    },
                    calibration_due_date: if calibration_due.is_empty() {
                        None
                    } else {
                        Some((*calibration_due).clone())
                    },
                };

                match client
                    .post::<_, serde_json::Value>("/meters", &new_meter)
                    .await
                {
                    Ok(_) => {
                        // Reset form
                        selected_building.set("".to_string());
                        selected_apartment.set("".to_string());
                        serial_number.set(String::default());
                        installation_date.set(String::default());
                        calibration_due.set(String::default());
                        apartments_list.set(Vec::new());
                        on_success.emit(());
                    }
                    Err(e) => {
                        on_error.emit(format!("Failed to register meter: {}", e));
                    }
                }
                submitting.set(false);
            });
        })
    };

    // Callbacks
    let on_building_change = {
        let selected_building = selected_building.clone();
        let selected_apartment = selected_apartment.clone();
        Callback::from(move |value: String| {
            selected_building.set(value);
            selected_apartment.set("".to_string());
        })
    };

    let on_apartment_change = {
        let selected_apartment = selected_apartment.clone();
        Callback::from(move |value: String| selected_apartment.set(value))
    };

    let on_meter_type_change = {
        let meter_type = meter_type.clone();
        Callback::from(move |value: String| meter_type.set(value))
    };

    let on_serial_change = {
        let serial_number = serial_number.clone();
        Callback::from(move |value: String| serial_number.set(value))
    };

    let on_installation_change = {
        let installation_date = installation_date.clone();
        Callback::from(move |value: String| installation_date.set(value))
    };

    let on_calibration_change = {
        let calibration_due = calibration_due.clone();
        Callback::from(move |value: String| calibration_due.set(value))
    };

    // Build options
    let building_options = {
        let mut options = vec![SelectOption::new("", "Select building...")];
        for building in buildings.iter() {
            options.push(SelectOption::new(
                building.id.to_string(),
                &building.address,
            ));
        }
        options
    };

    let apartment_options = {
        let mut options = vec![SelectOption::new("", "Select apartment...")];
        for apartment in apartments.iter() {
            options.push(SelectOption::new(
                apartment.id.to_string(),
                &apartment.number,
            ));
        }
        options
    };

    let meter_type_options = vec![
        SelectOption::new("ColdWater", "Cold Water"),
        SelectOption::new("HotWater", "Hot Water"),
        SelectOption::new("Gas", "Gas"),
        SelectOption::new("Electricity", "Electricity"),
    ];

    html! {
        <div class="card">
            <div class="card-body">
                <form onsubmit={on_submit}>
                    <FormGroup
                        title="Meter Location"
                        description="Select the building and apartment for this meter"
                    >
                        <div class="row">
                            <div class="col-md-6">
                                <Select
                                    label="Building"
                                    value={(*selected_building).clone()}
                                    on_change={on_building_change}
                                    options={building_options}
                                    required=true
                                />
                            </div>
                            <div class="col-md-6">
                                <Select
                                    label="Apartment"
                                    value={(*selected_apartment).clone()}
                                    on_change={on_apartment_change}
                                    options={apartment_options}
                                    disabled={selected_building.is_empty()}
                                    required=true
                                />
                            </div>
                        </div>
                    </FormGroup>

                    <FormGroup
                        title="Meter Details"
                        description="Enter the meter type and serial number"
                    >
                        <div class="row">
                            <div class="col-md-6">
                                <Select
                                    label="Meter Type"
                                    value={(*meter_type).clone()}
                                    on_change={on_meter_type_change}
                                    options={meter_type_options}
                                    required=true
                                />
                            </div>
                            <div class="col-md-6">
                                <TextInput
                                    label="Serial Number"
                                    value={(*serial_number).clone()}
                                    on_change={on_serial_change}
                                    placeholder="Enter meter serial number"
                                    disabled={*submitting}
                                    required=true
                                />
                            </div>
                        </div>
                    </FormGroup>

                    <FormGroup
                        title="Dates (Optional)"
                        description="Set installation and calibration dates"
                    >
                        <div class="row">
                            <div class="col-md-6">
                                <DateTimeInput
                                    label="Installation Date"
                                    value={(*installation_date).clone()}
                                    on_change={on_installation_change}
                                    input_type="date"
                                    disabled={*submitting}
                                />
                            </div>
                            <div class="col-md-6">
                                <DateTimeInput
                                    label="Calibration Due Date"
                                    value={(*calibration_due).clone()}
                                    on_change={on_calibration_change}
                                    input_type="date"
                                    disabled={*submitting}
                                />
                            </div>
                        </div>
                    </FormGroup>

                    <button type="submit" class="btn btn-primary" disabled={*submitting}>
                        if *submitting {
                            <span class="spinner-border spinner-border-sm me-2"></span>
                        }
                        {"Register Meter"}
                    </button>
                </form>
            </div>
        </div>
    }
}
