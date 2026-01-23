use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct Meter {
    id: u64,
    apartment_id: u64,
    meter_type: String,
    serial_number: String,
    installation_date: Option<String>,
    calibration_due_date: Option<String>,
    last_calibration_date: Option<String>,
    is_active: bool,
}

#[function_component(MeterCalibrationPage)]
pub fn meter_calibration_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    // Access control
    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    {"Access denied. Only Admins and Managers can view the calibration dashboard."}
                </div>
            </div>
        };
    }

    let meters = use_state(|| Vec::<Meter>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);
    let days_before = use_state(|| 30i64);

    let token = auth.token().map(|t| t.to_string());

    // Load meters needing calibration
    {
        let meters = meters.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();
        let days = *days_before;

        use_effect_with(days, move |d| {
            let d = *d;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<Vec<Meter>>(&format!("/meters/calibration-due?days_before={}", d))
                    .await
                {
                    Ok(list) => {
                        meters.set(list);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load meters: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_days_change = {
        let days_before = days_before.clone();
        let loading = loading.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            if let Ok(days) = select.value().parse::<i64>() {
                days_before.set(days);
                loading.set(true);
            }
        })
    };

    let on_meter_click = {
        let navigator = navigator.clone();
        Callback::from(move |meter_id: u64| navigator.push(&Route::MeterDetail { id: meter_id }))
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
                <h2>{"Calibration Dashboard"}</h2>
                <div class="d-flex align-items-center gap-2">
                    <label class="form-label mb-0">{"Show meters due within:"}</label>
                    <select class="form-select" style="width: auto;" value={days_before.to_string()} onchange={on_days_change}>
                        <option value="7">{"7 days"}</option>
                        <option value="30">{"30 days"}</option>
                        <option value="60">{"60 days"}</option>
                        <option value="90">{"90 days"}</option>
                        <option value="365">{"1 year"}</option>
                    </select>
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
            } else if meters.is_empty() {
                <div class="alert alert-success">
                    <i class="bi bi-check-circle"></i>
                    {" No meters requiring calibration in the selected time period."}
                </div>
            } else {
                <div class="alert alert-info mb-4">
                    <i class="bi bi-info-circle"></i>
                    {format!(" {} meter(s) requiring calibration within {} days", meters.len(), *days_before)}
                </div>

                <div class="row">
                    { for meters.iter().map(|meter| {
                        let meter_id = meter.id;
                        let on_click = {
                            let on_meter_click = on_meter_click.clone();
                            Callback::from(move |_| on_meter_click.emit(meter_id))
                        };

                        // Calculate days until calibration due
                        let (status_class, status_text, days_text) = if let Some(ref due_date) = meter.calibration_due_date {
                            // Parse date string (YYYY-MM-DD) and calculate days until due
                            let parts: Vec<&str> = due_date.split('-').collect();
                            if parts.len() == 3 {
                                if let (Ok(year), Ok(month), Ok(day)) = (
                                    parts[0].parse::<i32>(),
                                    parts[1].parse::<u32>(),
                                    parts[2].parse::<u32>()
                                ) {
                                    // Get current date from JavaScript
                                    let now = js_sys::Date::new_0();
                                    let today_ms = now.get_time();

                                    // Create due date
                                    let due_date_js = js_sys::Date::new_0();
                                    due_date_js.set_full_year(year as u32);
                                    due_date_js.set_month(month - 1); // JS months are 0-indexed
                                    due_date_js.set_date(day);
                                    let due_ms = due_date_js.get_time();

                                    // Calculate days difference
                                    let diff_ms = due_ms - today_ms;
                                    let days_until = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;

                                    if days_until < 0 {
                                        ("card border-danger", "badge bg-danger", format!("OVERDUE by {} days", -days_until))
                                    } else if days_until <= 7 {
                                        ("card border-danger", "badge bg-danger", format!("Due in {} days", days_until))
                                    } else if days_until <= 30 {
                                        ("card border-warning", "badge bg-warning text-dark", format!("Due in {} days", days_until))
                                    } else {
                                        ("card border-info", "badge bg-info", format!("Due in {} days", days_until))
                                    }
                                } else {
                                    ("card", "badge bg-secondary", "Unknown".to_string())
                                }
                            } else {
                                ("card", "badge bg-secondary", "Unknown".to_string())
                            }
                        } else {
                            ("card", "badge bg-secondary", "Not set".to_string())
                        };

                        html! {
                            <div class="col-md-6 col-lg-4 mb-3" key={meter.id}>
                                <div class={status_class} style="cursor: pointer;" onclick={on_click}>
                                    <div class="card-body">
                                        <div class="d-flex justify-content-between align-items-start mb-2">
                                            <h5 class="card-title mb-0">
                                                { &meter.meter_type }
                                            </h5>
                                            <span class={status_text}>
                                                { days_text }
                                            </span>
                                        </div>

                                        <p class="card-text text-muted mb-2">
                                            <small>{"Serial: "}{&meter.serial_number}</small>
                                        </p>

                                        <div class="mt-3 pt-3 border-top">
                                            if let Some(ref due_date) = meter.calibration_due_date {
                                                <p class="mb-1">
                                                    <strong>{"Calibration Due:"}</strong>
                                                    <br/>
                                                    {due_date}
                                                </p>
                                            }
                                            if let Some(ref last_cal) = meter.last_calibration_date {
                                                <p class="mb-1 text-muted">
                                                    <small>{"Last Calibrated: "}{last_cal}</small>
                                                </p>
                                            }
                                            if let Some(ref inst_date) = meter.installation_date {
                                                <p class="mb-0 text-muted">
                                                    <small>{"Installed: "}{inst_date}</small>
                                                </p>
                                            }
                                        </div>

                                        <div class="mt-3">
                                            <small class="text-muted">
                                                {"Click to view meter details and record calibration"}
                                            </small>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }) }
                </div>

                <div class="alert alert-light mt-4">
                    <h6>{"Legend:"}</h6>
                    <div class="d-flex flex-wrap gap-3">
                        <div>
                            <span class="badge bg-danger">{"Overdue / Due in ≤7 days"}</span>
                            {" - Immediate action required"}
                        </div>
                        <div>
                            <span class="badge bg-warning text-dark">{"Due in 8-30 days"}</span>
                            {" - Schedule calibration soon"}
                        </div>
                        <div>
                            <span class="badge bg-info">{"Due in >30 days"}</span>
                            {" - Monitor"}
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}
