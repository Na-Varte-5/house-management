use crate::i18n::{t, t_with_args};
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct MeterWithLastReading {
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
    pub last_reading_value: Option<String>,
    pub last_reading_timestamp: Option<String>,
    pub last_reading_unit: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct MeterCardListProps {
    pub meters: Vec<MeterWithLastReading>,
    pub on_meter_click: Callback<u64>,
}

#[function_component(MeterCardList)]
pub fn meter_card_list(props: &MeterCardListProps) -> Html {
    if props.meters.is_empty() {
        return html! {
            <div class="alert alert-info">
                <i class="bi bi-info-circle me-2"></i>
                {t("meters-no-meters-apartment")}
            </div>
        };
    }

    html! {
        <div class="row">
            { for props.meters.iter().map(|meter| {
                let meter_id = meter.id;
                let on_click = {
                    let on_meter_click = props.on_meter_click.clone();
                    Callback::from(move |_| on_meter_click.emit(meter_id))
                };

                let (cal_badge, cal_text) = calibration_status(&meter.calibration_due_date);

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
                                    <small>{t("meters-serial-label")}{" "}{&meter.serial_number}</small>
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
                                        {t("meters-installed-label")}{" "}{inst_date}
                                    </div>
                                }
                            </div>
                        </div>
                    </div>
                }
            }) }
        </div>
    }
}

fn calibration_status(calibration_due_date: &Option<String>) -> (&'static str, String) {
    if let Some(due_date) = calibration_due_date {
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
                let days_until = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;

                if days_until < 0 {
                    return (
                        "badge bg-danger",
                        t_with_args(
                            "meters-overdue-by-days",
                            &[("days", &(-days_until).to_string())],
                        ),
                    );
                } else if days_until <= 30 {
                    return (
                        "badge bg-warning text-dark",
                        t_with_args("meters-due-in-days", &[("days", &days_until.to_string())]),
                    );
                } else {
                    return (
                        "badge bg-success",
                        t_with_args("meters-valid-days", &[("days", &days_until.to_string())]),
                    );
                }
            }
        }
        ("badge bg-secondary", t("meters-unknown"))
    } else {
        ("badge bg-secondary", t("properties-not-set"))
    }
}
