use yew::prelude::*;
use yew_router::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use crate::components::{AdminLayout, ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

#[derive(PartialEq, Clone)]
enum Tab {
    List,
    Register,
}

#[derive(Deserialize, Clone, PartialEq)]
struct Building {
    id: u64,
    address: String,
}

#[derive(Deserialize, Clone, PartialEq)]
struct Apartment {
    id: u64,
    number: String,
    building_id: u64,
}

#[derive(Serialize)]
struct NewMeter {
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct MeterWithApartment {
    id: u64,
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
    last_calibration_date: Option<String>,
    is_active: bool,
    apartment_number: Option<String>,
    building_id: Option<u64>,
    building_address: Option<String>,
    last_reading_value: Option<String>,
    last_reading_timestamp: Option<String>,
    last_reading_unit: Option<String>,
}

#[function_component(MeterManagementPage)]
pub fn meter_management_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    {"Access denied. Only Admins and Managers can access meter management."}
                </div>
            </div>
        };
    }

    let active_tab = use_state(|| Tab::List);
    let token = auth.token().map(|t| t.to_string());

    // Register meter state
    let buildings = use_state(|| Vec::<Building>::new());
    let apartments = use_state(|| Vec::<Apartment>::new());
    let selected_building = use_state(|| None::<u64>);
    let selected_apartment = use_state(|| None::<u64>);
    let meter_type = use_state(|| "ColdWater".to_string());
    let serial_number = use_state(String::default);
    let installation_date = use_state(String::default);
    let calibration_due = use_state(String::default);
    let submitting = use_state(|| false);

    // List meters state
    let all_meters = use_state(|| Vec::<MeterWithApartment>::new());
    let all_apartments = use_state(|| Vec::<Apartment>::new());
    let loading = use_state(|| false);
    let search_query = use_state(String::default);
    let filter_building = use_state(|| None::<u64>);
    let filter_calibration = use_state(|| "all".to_string());

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Load buildings for both tabs
    {
        let buildings = buildings.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Building>>("/buildings").await {
                    Ok(list) => buildings.set(list),
                    Err(e) => error.set(Some(format!("Failed to load buildings: {}", e))),
                }
            });
            || ()
        });
    }

    // Load apartments when building selected (for register tab)
    {
        let apartments = apartments.clone();
        let selected_building = selected_building.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((*selected_building,), move |(building_id,)| {
            if let Some(id) = *building_id {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    match client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", id)).await {
                        Ok(list) => apartments.set(list),
                        Err(e) => error.set(Some(format!("Failed to load apartments: {}", e))),
                    }
                });
            } else {
                apartments.set(Vec::new());
            }
            || ()
        });
    }

    // Load all apartments for meter list (for building mapping)
    {
        let all_apartments = all_apartments.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Apartment>>("/apartments").await {
                    Ok(list) => all_apartments.set(list),
                    Err(e) => error.set(Some(format!("Failed to load apartments: {}", e))),
                }
            });
            || ()
        });
    }

    // Load all meters when list tab is active
    {
        let all_meters = all_meters.clone();
        let all_apartments = all_apartments.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();
        let active_tab = active_tab.clone();

        use_effect_with((*active_tab).clone(), move |tab| {
            if matches!(tab, Tab::List) {
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());

                    // Fetch all buildings and their apartments with meters
                    match client.get::<Vec<Building>>("/buildings").await {
                        Ok(buildings_list) => {
                            let mut all_meters_list = Vec::new();

                            for building in buildings_list {
                                if let Ok(apartments_list) = client.get::<Vec<Apartment>>(&format!("/buildings/{}/apartments", building.id)).await {
                                    for apartment in apartments_list {
                                        // Fetch meters for this apartment
                                        if let Ok(meters) = client.get::<Vec<serde_json::Value>>(&format!("/apartments/{}/meters", apartment.id)).await {
                                            for meter_json in meters {
                                                if let Ok(mut meter) = serde_json::from_value::<MeterWithApartment>(meter_json.clone()) {
                                                    meter.apartment_number = Some(apartment.number.clone());
                                                    meter.building_id = Some(building.id);
                                                    meter.building_address = Some(building.address.clone());
                                                    all_meters_list.push(meter);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            all_meters.set(all_meters_list);
                            loading.set(false);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load meters: {}", e)));
                            loading.set(false);
                        }
                    }
                });
            }
            || ()
        });
    }

    let on_tab_change = {
        let active_tab = active_tab.clone();
        let error = error.clone();
        let success = success.clone();
        move |tab: Tab| {
            error.set(None);
            success.set(None);
            active_tab.set(tab);
        }
    };

    let on_building_change = {
        let selected_building = selected_building.clone();
        let selected_apartment = selected_apartment.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            if let Ok(id) = select.value().parse::<u64>() {
                selected_building.set(Some(id));
                selected_apartment.set(None);
            } else {
                selected_building.set(None);
                selected_apartment.set(None);
            }
        })
    };

    let on_apartment_change = {
        let selected_apartment = selected_apartment.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            if let Ok(id) = select.value().parse::<u64>() {
                selected_apartment.set(Some(id));
            }
        })
    };

    let on_submit = {
        let selected_apartment = selected_apartment.clone();
        let meter_type = meter_type.clone();
        let serial_number = serial_number.clone();
        let installation_date = installation_date.clone();
        let calibration_due = calibration_due.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if selected_apartment.is_none() {
                error.set(Some("Please select an apartment".to_string()));
                return;
            }

            if serial_number.trim().is_empty() {
                error.set(Some("Serial number is required".to_string()));
                return;
            }

            submitting.set(true);
            error.set(None);

            let new_meter = NewMeter {
                apartment_id: selected_apartment.unwrap(),
                meter_type: (*meter_type).clone(),
                serial_number: (*serial_number).clone(),
                installation_date: if installation_date.is_empty() { None } else { Some((*installation_date).clone()) },
                calibration_due_date: if calibration_due.is_empty() { None } else { Some((*calibration_due).clone()) },
            };

            let submitting = submitting.clone();
            let error = error.clone();
            let success = success.clone();
            let serial_number = serial_number.clone();
            let installation_date = installation_date.clone();
            let calibration_due = calibration_due.clone();
            let token = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.post::<_, serde_json::Value>("/meters", &new_meter).await {
                    Ok(_) => {
                        success.set(Some("Meter registered successfully".to_string()));
                        submitting.set(false);
                        serial_number.set(String::default());
                        installation_date.set(String::default());
                        calibration_due.set(String::default());
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to register meter: {}", e)));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_meter_click = {
        let navigator = navigator.clone();
        Callback::from(move |meter_id: u64| {
            navigator.push(&Route::MeterDetail { id: meter_id })
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

    // Filter meters based on search and filters
    let filtered_meters = {
        let query = (*search_query).clone().to_lowercase();
        let building_filter = *filter_building;
        let calibration_filter = (*filter_calibration).clone();

        all_meters.iter().filter(|meter| {
            // Search filter
            let matches_search = query.is_empty() ||
                meter.serial_number.to_lowercase().contains(&query) ||
                meter.meter_type.to_lowercase().contains(&query) ||
                meter.apartment_number.as_ref().map(|n| n.to_lowercase().contains(&query)).unwrap_or(false) ||
                meter.building_address.as_ref().map(|b| b.to_lowercase().contains(&query)).unwrap_or(false);

            // Building filter
            let matches_building = building_filter.is_none() ||
                meter.building_id == building_filter;

            // Calibration filter
            let matches_calibration = match calibration_filter.as_str() {
                "overdue" => {
                    meter.calibration_due_date.as_ref().map(|due_date| {
                        let parts: Vec<&str> = due_date.split('-').collect();
                        if parts.len() == 3 {
                            if let (Ok(year), Ok(month), Ok(day)) = (
                                parts[0].parse::<i32>(),
                                parts[1].parse::<u32>(),
                                parts[2].parse::<u32>()
                            ) {
                                let now = js_sys::Date::new_0();
                                let today_ms = now.get_time();
                                let due_date_js = js_sys::Date::new_0();
                                due_date_js.set_full_year(year as u32);
                                due_date_js.set_month(month - 1);
                                due_date_js.set_date(day);
                                let due_ms = due_date_js.get_time();
                                due_ms < today_ms
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }).unwrap_or(false)
                }
                "due_soon" => {
                    meter.calibration_due_date.as_ref().map(|due_date| {
                        let parts: Vec<&str> = due_date.split('-').collect();
                        if parts.len() == 3 {
                            if let (Ok(year), Ok(month), Ok(day)) = (
                                parts[0].parse::<i32>(),
                                parts[1].parse::<u32>(),
                                parts[2].parse::<u32>()
                            ) {
                                let now = js_sys::Date::new_0();
                                let today_ms = now.get_time();
                                let due_date_js = js_sys::Date::new_0();
                                due_date_js.set_full_year(year as u32);
                                due_date_js.set_month(month - 1);
                                due_date_js.set_date(day);
                                let due_ms = due_date_js.get_time();
                                let diff_ms = due_ms - today_ms;
                                let days_until = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;
                                days_until >= 0 && days_until <= 30
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }).unwrap_or(false)
                }
                "valid" => {
                    meter.calibration_due_date.as_ref().map(|due_date| {
                        let parts: Vec<&str> = due_date.split('-').collect();
                        if parts.len() == 3 {
                            if let (Ok(year), Ok(month), Ok(day)) = (
                                parts[0].parse::<i32>(),
                                parts[1].parse::<u32>(),
                                parts[2].parse::<u32>()
                            ) {
                                let now = js_sys::Date::new_0();
                                let today_ms = now.get_time();
                                let due_date_js = js_sys::Date::new_0();
                                due_date_js.set_full_year(year as u32);
                                due_date_js.set_month(month - 1);
                                due_date_js.set_date(day);
                                let due_ms = due_date_js.get_time();
                                let diff_ms = due_ms - today_ms;
                                let days_until = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;
                                days_until > 30
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }).unwrap_or(false)
                }
                _ => true
            };

            matches_search && matches_building && matches_calibration
        }).cloned().collect::<Vec<_>>()
    };

    html! {
        <AdminLayout title="Meter Management" active_route={Route::MeterManagement}>
            // Tabs
            <ul class="nav nav-tabs mb-3">
                <li class="nav-item">
                    <a
                        class={if matches!(*active_tab, Tab::List) { "nav-link active" } else { "nav-link" }}
                        style="cursor: pointer;"
                        onclick={let on_tab = on_tab_change.clone(); Callback::from(move |_| on_tab(Tab::List))}
                    >
                        {"List Meters"}
                    </a>
                </li>
                <li class="nav-item">
                    <a
                        class={if matches!(*active_tab, Tab::Register) { "nav-link active" } else { "nav-link" }}
                        style="cursor: pointer;"
                        onclick={let on_tab = on_tab_change.clone(); Callback::from(move |_| on_tab(Tab::Register))}
                    >
                        {"Register Meter"}
                    </a>
                </li>
            </ul>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            // Tab content
            if matches!(*active_tab, Tab::List) {
                // Meter List Table
                <div>
                    // Filters
                    <div class="card mb-3">
                        <div class="card-body">
                            <div class="row g-3">
                                <div class="col-md-4">
                                    <label class="form-label small">{"Search"}</label>
                                    <input
                                        type="text"
                                        class="form-control form-control-sm"
                                        placeholder="Search by building, apartment, type, serial..."
                                        value={(*search_query).clone()}
                                        oninput={{
                                            let search_query = search_query.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                search_query.set(input.value());
                                            })
                                        }}
                                    />
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label small">{"Building"}</label>
                                    <select
                                        class="form-select form-select-sm"
                                        onchange={{
                                            let filter_building = filter_building.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                let val = select.value();
                                                if val.is_empty() {
                                                    filter_building.set(None);
                                                } else if let Ok(id) = val.parse::<u64>() {
                                                    filter_building.set(Some(id));
                                                }
                                            })
                                        }}
                                    >
                                        <option value="" selected={filter_building.is_none()}>{"All buildings"}</option>
                                        { for buildings.iter().map(|b| {
                                            let is_selected = *filter_building == Some(b.id);
                                            html! {
                                                <option value={b.id.to_string()} selected={is_selected}>{&b.address}</option>
                                            }
                                        })}
                                    </select>
                                </div>
                                <div class="col-md-4">
                                    <label class="form-label small">{"Calibration Status"}</label>
                                    <select
                                        class="form-select form-select-sm"
                                        value={(*filter_calibration).clone()}
                                        onchange={{
                                            let filter_calibration = filter_calibration.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                filter_calibration.set(select.value());
                                            })
                                        }}
                                    >
                                        <option value="all">{"All meters"}</option>
                                        <option value="overdue">{"Overdue"}</option>
                                        <option value="due_soon">{"Due within 30 days"}</option>
                                        <option value="valid">{"Valid (>30 days)"}</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>

                    if *loading {
                        <div class="text-center py-5">
                            <div class="spinner-border" role="status">
                                <span class="visually-hidden">{"Loading..."}</span>
                            </div>
                        </div>
                    } else if filtered_meters.is_empty() {
                        <div class="alert alert-info">
                            {"No meters found matching your filters."}
                        </div>
                    } else {
                        <>
                            <div class="mb-2 text-muted small">
                                {format!("Showing {} meter(s)", filtered_meters.len())}
                            </div>
                            <div class="table-responsive">
                                <table class="table table-hover">
                                    <thead>
                                        <tr>
                                            <th>{"Building"}</th>
                                            <th>{"Apartment"}</th>
                                            <th>{"Type"}</th>
                                            <th>{"Serial Number"}</th>
                                            <th>{"Last Reading"}</th>
                                            <th>{"Installation Date"}</th>
                                            <th>{"Calibration Due"}</th>
                                            <th>{"Cal. Status"}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for filtered_meters.iter().map(|meter| {
                                            let meter_id = meter.id;
                                            let on_click = {
                                                let on_meter_click = on_meter_click.clone();
                                                Callback::from(move |_| on_meter_click.emit(meter_id))
                                            };

                                            // Calculate calibration status
                                            let (badge_class, badge_text) = if let Some(ref due_date) = meter.calibration_due_date {
                                                let parts: Vec<&str> = due_date.split('-').collect();
                                                if parts.len() == 3 {
                                                    if let (Ok(year), Ok(month), Ok(day)) = (
                                                        parts[0].parse::<i32>(),
                                                        parts[1].parse::<u32>(),
                                                        parts[2].parse::<u32>()
                                                    ) {
                                                        let now = js_sys::Date::new_0();
                                                        let today_ms = now.get_time();
                                                        let due_date_js = js_sys::Date::new_0();
                                                        due_date_js.set_full_year(year as u32);
                                                        due_date_js.set_month(month - 1);
                                                        due_date_js.set_date(day);
                                                        let due_ms = due_date_js.get_time();
                                                        let diff_ms = due_ms - today_ms;
                                                        let days_until = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;

                                                        if days_until < 0 {
                                                            ("badge bg-danger", format!("Overdue ({} days)", -days_until))
                                                        } else if days_until <= 30 {
                                                            ("badge bg-warning text-dark", format!("Due in {} days", days_until))
                                                        } else {
                                                            ("badge bg-success", format!("Valid ({} days)", days_until))
                                                        }
                                                    } else {
                                                        ("badge bg-secondary", "Unknown".to_string())
                                                    }
                                                } else {
                                                    ("badge bg-secondary", "Unknown".to_string())
                                                }
                                            } else {
                                                ("badge bg-secondary", "Not set".to_string())
                                            };

                                            // Format last reading
                                            let last_reading_display = if let Some(ref value) = meter.last_reading_value {
                                                let unit = meter.last_reading_unit.as_ref().map(|u| u.as_str()).unwrap_or("");
                                                format!("{} {}", value, unit)
                                            } else {
                                                "No readings".to_string()
                                            };

                                            html! {
                                                <tr key={meter.id} style="cursor: pointer;" onclick={on_click}>
                                                    <td>{meter.building_address.as_ref().unwrap_or(&"-".to_string())}</td>
                                                    <td>{meter.apartment_number.as_ref().unwrap_or(&"-".to_string())}</td>
                                                    <td>{&meter.meter_type}</td>
                                                    <td><code>{&meter.serial_number}</code></td>
                                                    <td>{last_reading_display}</td>
                                                    <td>{meter.installation_date.as_ref().unwrap_or(&"-".to_string())}</td>
                                                    <td>{meter.calibration_due_date.as_ref().unwrap_or(&"-".to_string())}</td>
                                                    <td><span class={badge_class}>{badge_text}</span></td>
                                                </tr>
                                            }
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </>
                    }
                </div>
            } else {
                <div class="card">
                    <div class="card-body">
                        <form onsubmit={on_submit}>
                            <div class="row">
                                <div class="col-md-6 mb-3">
                                    <label class="form-label">{"Building"}</label>
                                    <select
                                        class="form-select"
                                        onchange={on_building_change}
                                        required=true
                                    >
                                        <option value="" selected={selected_building.is_none()}>{"Select building..."}</option>
                                        { for buildings.iter().map(|b| {
                                            let is_selected = *selected_building == Some(b.id);
                                            html! {
                                                <option value={b.id.to_string()} selected={is_selected}>{&b.address}</option>
                                            }
                                        })}
                                    </select>
                                </div>
                                <div class="col-md-6 mb-3">
                                    <label class="form-label">{"Apartment"}</label>
                                    <select
                                        class="form-select"
                                        onchange={on_apartment_change}
                                        disabled={selected_building.is_none()}
                                        required=true
                                    >
                                        <option value="" selected={selected_apartment.is_none()}>{"Select apartment..."}</option>
                                        { for apartments.iter().map(|a| {
                                            let is_selected = *selected_apartment == Some(a.id);
                                            html! {
                                                <option value={a.id.to_string()} selected={is_selected}>{&a.number}</option>
                                            }
                                        })}
                                    </select>
                                </div>
                            </div>

                            <div class="row">
                                <div class="col-md-6 mb-3">
                                    <label class="form-label">{"Meter Type"}</label>
                                    <select class="form-select" value={(*meter_type).clone()} onchange={{
                                        let meter_type = meter_type.clone();
                                        Callback::from(move |e: Event| {
                                            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                            meter_type.set(select.value());
                                        })
                                    }}>
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
                                        value={(*serial_number).clone()}
                                        oninput={{
                                            let serial_number = serial_number.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                serial_number.set(input.value());
                                            })
                                        }}
                                        required=true
                                    />
                                </div>
                            </div>

                            <div class="row">
                                <div class="col-md-6 mb-3">
                                    <label class="form-label">{"Installation Date (optional)"}</label>
                                    <input
                                        type="date"
                                        class="form-control"
                                        value={(*installation_date).clone()}
                                        oninput={{
                                            let installation_date = installation_date.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                installation_date.set(input.value());
                                            })
                                        }}
                                    />
                                </div>
                                <div class="col-md-6 mb-3">
                                    <label class="form-label">{"Calibration Due Date (optional)"}</label>
                                    <input
                                        type="date"
                                        class="form-control"
                                        value={(*calibration_due).clone()}
                                        oninput={{
                                            let calibration_due = calibration_due.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                calibration_due.set(input.value());
                                            })
                                        }}
                                    />
                                </div>
                            </div>

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
        </AdminLayout>
    }
}

