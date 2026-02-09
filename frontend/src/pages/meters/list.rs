use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::i18n::{t, t_with_args};
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct MeterWithLastReading {
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
    last_reading_value: Option<String>,
    last_reading_timestamp: Option<String>,
    last_reading_unit: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub apartment_id: u64,
}

#[function_component(MeterListPage)]
pub fn meter_list_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let meters = use_state(|| Vec::<MeterWithLastReading>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let apartment_id = props.apartment_id;
    let token = auth.token().map(|t| t.to_string());
    let is_admin_or_manager = auth.is_admin_or_manager();

    // Load meters on mount
    {
        let meters = meters.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(apartment_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<Vec<MeterWithLastReading>>(&format!("/apartments/{}/meters", id))
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

    let on_new_meter = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::MeterNew))
    };

    let on_meter_click = {
        let navigator = navigator.clone();
        Callback::from(move |meter_id: u64| navigator.push(&Route::MeterDetail { id: meter_id }))
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.back())
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
                        <i class="bi bi-arrow-left"></i> {t("meters-back")}
                    </button>
                    <h2 class="d-inline">{t("meters-title")}</h2>
                </div>
                if is_admin_or_manager {
                    <button class="btn btn-primary" onclick={on_new_meter}>
                        {t("meters-register-new-btn")}
                    </button>
                }
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
                        <span class="visually-hidden">{t("loading")}</span>
                    </div>
                </div>
            } else if meters.is_empty() {
                <div class="alert alert-info">
                    {t("meters-no-meters")}
                    if is_admin_or_manager {
                        {" "}{t("meters-empty-state-hint")}
                    }
                </div>
            } else {
                <div class="row">
                    { for meters.iter().map(|meter| {
                        let meter_id = meter.id;
                        let on_click = {
                            let on_meter_click = on_meter_click.clone();
                            Callback::from(move |_| on_meter_click.emit(meter_id))
                        };

                        // Determine calibration status
                        let (cal_badge, cal_text) = if let Some(ref due_date) = meter.calibration_due_date {
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
                                        ("badge bg-danger", t_with_args("meters-overdue-by-days", &[("days", &(-days_until).to_string())]))
                                    } else if days_until <= 30 {
                                        ("badge bg-warning text-dark", t_with_args("meters-due-in-days", &[("days", &days_until.to_string())]))
                                    } else {
                                        ("badge bg-success", t_with_args("meters-valid-days", &[("days", &days_until.to_string())]))
                                    }
                                } else {
                                    ("badge bg-secondary", t("meters-unknown"))
                                }
                            } else {
                                ("badge bg-secondary", t("meters-unknown"))
                            }
                        } else {
                            ("badge bg-secondary", t("meters-not-set"))
                        };

                        html! {
                            <div class="col-md-6 col-lg-4 mb-3" key={meter.id}>
                                <div class="card h-100 shadow-sm" style="cursor: pointer;" onclick={on_click}>
                                    <div class="card-body">
                                        <div class="d-flex justify-content-between align-items-start mb-2">
                                            <h5 class="card-title mb-0">
                                                { &meter.meter_type }
                                            </h5>
                                            <span class={cal_badge}>
                                                { cal_text }
                                            </span>
                                        </div>
                                        <p class="card-text text-muted mb-2">
                                            <small>{t("meters-serial-label")}{&meter.serial_number}</small>
                                        </p>

                                        if let Some(ref last_value) = meter.last_reading_value {
                                            <div class="mt-3 pt-3 border-top">
                                                <div class="d-flex justify-content-between">
                                                    <span class="text-muted">{t("meters-last-reading-label")}</span>
                                                    <strong>
                                                        { last_value }
                                                        {" "}
                                                        { meter.last_reading_unit.as_ref().unwrap_or(&"".to_string()) }
                                                    </strong>
                                                </div>
                                                if let Some(ref timestamp) = meter.last_reading_timestamp {
                                                    <div class="text-muted small mt-1">
                                                        { timestamp }
                                                    </div>
                                                }
                                            </div>
                                        } else {
                                            <div class="mt-3 pt-3 border-top text-muted">
                                                <em>{t("meters-no-readings-short")}</em>
                                            </div>
                                        }

                                        if let Some(ref inst_date) = meter.installation_date {
                                            <div class="mt-2 text-muted small">
                                                {t("meters-installed-label")}{inst_date}
                                            </div>
                                        }
                                    </div>
                                </div>
                            </div>
                        }
                    }) }
                </div>
            }
        </div>
    }
}
