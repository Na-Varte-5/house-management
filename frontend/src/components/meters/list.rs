use crate::components::{Select, SelectOption, TextInput};
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct Building {
    pub id: u64,
    pub address: String,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct MeterWithApartment {
    pub id: u64,
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
    pub last_calibration_date: Option<String>,
    pub is_active: bool,
    pub apartment_number: Option<String>,
    pub building_id: Option<u64>,
    pub building_address: Option<String>,
    pub last_reading_value: Option<String>,
    pub last_reading_timestamp: Option<String>,
    pub last_reading_unit: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct MeterListProps {
    /// List of all buildings for filter dropdown
    pub buildings: Vec<Building>,

    /// Optional token for API authentication
    pub token: Option<String>,

    /// Whether to reload meters (triggers load when changed)
    pub reload_trigger: u32,

    /// Callback to set error message
    pub on_error: Callback<String>,
}

#[function_component(MeterList)]
pub fn meter_list(props: &MeterListProps) -> Html {
    let navigator = use_navigator().unwrap();

    let all_meters = use_state(|| Vec::<MeterWithApartment>::new());
    let loading = use_state(|| false);
    let search_query = use_state(String::default);
    let filter_building = use_state(|| "".to_string());
    let filter_calibration = use_state(|| "all".to_string());

    // Load meters
    {
        let all_meters = all_meters.clone();
        let loading = loading.clone();
        let on_error = props.on_error.clone();
        let token = props.token.clone();
        let reload_trigger = props.reload_trigger;

        use_effect_with(reload_trigger, move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<MeterWithApartment>>("/meters").await {
                    Ok(list) => all_meters.set(list),
                    Err(e) => on_error.emit(format!("Failed to load meters: {}", e)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_meter_click = {
        let navigator = navigator.clone();
        Callback::from(move |meter_id: u64| navigator.push(&Route::MeterDetail { id: meter_id }))
    };

    // Callbacks for filters
    let on_search_change = {
        let search_query = search_query.clone();
        Callback::from(move |value: String| search_query.set(value))
    };

    let on_filter_building_change = {
        let filter_building = filter_building.clone();
        Callback::from(move |value: String| filter_building.set(value))
    };

    let on_filter_calibration_change = {
        let filter_calibration = filter_calibration.clone();
        Callback::from(move |value: String| filter_calibration.set(value))
    };

    // Build filter options
    let filter_building_options = {
        let mut options = vec![SelectOption::new("", "All Buildings")];
        for building in props.buildings.iter() {
            options.push(SelectOption::new(
                building.id.to_string(),
                &building.address,
            ));
        }
        options
    };

    let filter_calibration_options = vec![
        SelectOption::new("all", "All Statuses"),
        SelectOption::new("overdue", "Overdue"),
        SelectOption::new("due_soon", "Due Soon (30 days)"),
        SelectOption::new("valid", "Valid"),
    ];

    // Filter meters based on search and filters
    let filtered_meters = {
        let query = (*search_query).clone().to_lowercase();
        let building_filter = if filter_building.is_empty() {
            None
        } else {
            filter_building.parse::<u64>().ok()
        };
        let calibration_filter = (*filter_calibration).clone();

        all_meters
            .iter()
            .filter(|meter| {
                // Search filter
                let matches_search = query.is_empty()
                    || meter.serial_number.to_lowercase().contains(&query)
                    || meter.meter_type.to_lowercase().contains(&query)
                    || meter
                        .apartment_number
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || meter
                        .building_address
                        .as_ref()
                        .map(|b| b.to_lowercase().contains(&query))
                        .unwrap_or(false);

                // Building filter
                let matches_building =
                    building_filter.is_none() || meter.building_id == building_filter;

                // Calibration filter
                let matches_calibration = match calibration_filter.as_str() {
                    "overdue" => meter
                        .calibration_due_date
                        .as_ref()
                        .map(|due_date| {
                            let parts: Vec<&str> = due_date.split('-').collect();
                            if parts.len() == 3 {
                                if let (Ok(year), Ok(month), Ok(day)) = (
                                    parts[0].parse::<i32>(),
                                    parts[1].parse::<u32>(),
                                    parts[2].parse::<u32>(),
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
                        })
                        .unwrap_or(false),
                    "due_soon" => meter
                        .calibration_due_date
                        .as_ref()
                        .map(|due_date| {
                            let parts: Vec<&str> = due_date.split('-').collect();
                            if parts.len() == 3 {
                                if let (Ok(year), Ok(month), Ok(day)) = (
                                    parts[0].parse::<i32>(),
                                    parts[1].parse::<u32>(),
                                    parts[2].parse::<u32>(),
                                ) {
                                    let now = js_sys::Date::new_0();
                                    let today_ms = now.get_time();
                                    let due_date_js = js_sys::Date::new_0();
                                    due_date_js.set_full_year(year as u32);
                                    due_date_js.set_month(month - 1);
                                    due_date_js.set_date(day);
                                    let due_ms = due_date_js.get_time();
                                    let diff_ms = due_ms - today_ms;
                                    let days_until =
                                        (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;
                                    days_until >= 0 && days_until <= 30
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false),
                    "valid" => meter
                        .calibration_due_date
                        .as_ref()
                        .map(|due_date| {
                            let parts: Vec<&str> = due_date.split('-').collect();
                            if parts.len() == 3 {
                                if let (Ok(year), Ok(month), Ok(day)) = (
                                    parts[0].parse::<i32>(),
                                    parts[1].parse::<u32>(),
                                    parts[2].parse::<u32>(),
                                ) {
                                    let now = js_sys::Date::new_0();
                                    let today_ms = now.get_time();
                                    let due_date_js = js_sys::Date::new_0();
                                    due_date_js.set_full_year(year as u32);
                                    due_date_js.set_month(month - 1);
                                    due_date_js.set_date(day);
                                    let due_ms = due_date_js.get_time();
                                    let diff_ms = due_ms - today_ms;
                                    let days_until =
                                        (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;
                                    days_until > 30
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false),
                    _ => true,
                };

                matches_search && matches_building && matches_calibration
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    html! {
        <div>
            // Filters
            <div class="card mb-3">
                <div class="card-body">
                    <div class="row">
                        <div class="col-md-4">
                            <TextInput
                                label="Search"
                                value={(*search_query).clone()}
                                on_change={on_search_change}
                                placeholder="Search by serial, type, or apartment..."
                            />
                        </div>
                        <div class="col-md-4">
                            <Select
                                label="Filter by Building"
                                value={(*filter_building).clone()}
                                on_change={on_filter_building_change}
                                options={filter_building_options}
                            />
                        </div>
                        <div class="col-md-4">
                            <Select
                                label="Calibration Status"
                                value={(*filter_calibration).clone()}
                                on_change={on_filter_calibration_change}
                                options={filter_calibration_options}
                            />
                        </div>
                    </div>
                </div>
            </div>

            // Meters table
            {
                if *loading {
                    html! {
                        <div class="text-center py-5">
                            <div class="spinner-border" role="status">
                                <span class="visually-hidden">{"Loading..."}</span>
                            </div>
                        </div>
                    }
                } else if filtered_meters.is_empty() {
                    html! {
                        <div class="alert alert-info">
                            {"No meters found. Register a meter to get started."}
                        </div>
                    }
                } else {
                    html! {
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
                    }
                }
            }
        </div>
    }
}
