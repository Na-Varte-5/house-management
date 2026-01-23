use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::{ErrorAlert, SuccessAlert};
use crate::components::meters::{
    ReadingEntryForm, MeterEditForm,
    ReadingHistory, MeterReading,
    Meter as MeterComponent
};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: u64,
}

#[function_component(MeterDetailPage)]
pub fn meter_detail_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let meter = use_state(|| None::<MeterComponent>);
    let readings = use_state(|| Vec::<MeterReading>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    // Form visibility state
    let show_entry_form = use_state(|| false);
    let show_edit_form = use_state(|| false);

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
                match client.get::<MeterComponent>(&format!("/meters/{}", id)).await {
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
        Callback::from(move |_: web_sys::MouseEvent| show_entry_form.set(!*show_entry_form))
    };

    let toggle_edit_form = {
        let show_edit_form = show_edit_form.clone();
        Callback::from(move |_: web_sys::MouseEvent| show_edit_form.set(!*show_edit_form))
    };

    let cancel_entry_form = {
        let show_entry_form = show_entry_form.clone();
        Callback::from(move |_| show_entry_form.set(false))
    };

    let cancel_edit_form = {
        let show_edit_form = show_edit_form.clone();
        Callback::from(move |_| show_edit_form.set(false))
    };

    let on_entry_success = {
        let success = success.clone();
        let show_entry_form = show_entry_form.clone();
        let readings = readings.clone();
        let token = token.clone();

        Callback::from(move |msg: String| {
            success.set(Some(msg));
            show_entry_form.set(false);

            // Reload readings
            let readings = readings.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client.get::<Vec<MeterReading>>(&format!("/meters/{}/readings", meter_id)).await {
                    readings.set(list);
                }
            });
        })
    };

    let on_edit_success = {
        let success = success.clone();
        let show_edit_form = show_edit_form.clone();
        let meter = meter.clone();

        Callback::from(move |updated_meter: MeterComponent| {
            success.set(Some("Meter updated successfully".to_string()));
            show_edit_form.set(false);
            meter.set(Some(updated_meter));
        })
    };

    let on_error = {
        let error = error.clone();
        Callback::from(move |msg: String| error.set(Some(msg)))
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
                        <ReadingEntryForm
                            meter_id={meter_id}
                            token={token.clone()}
                            on_success={on_entry_success}
                            on_error={on_error.clone()}
                            on_cancel={cancel_entry_form}
                        />
                    }

                    // Edit/Replace meter form
                    if *show_edit_form && is_admin_or_manager {
                        <MeterEditForm
                            meter={m.clone()}
                            token={token.clone()}
                            on_success={on_edit_success}
                            on_error={on_error}
                            on_cancel={cancel_edit_form}
                        />
                    }

                    // Readings table component
                    <ReadingHistory
                        meter_id={meter_id}
                        readings={(*readings).clone()}
                        on_export={on_export_csv}
                    />
                </div>
            } else {
                <div class="alert alert-danger">
                    {"Meter not found"}
                </div>
            }
        </div>
    }
}
