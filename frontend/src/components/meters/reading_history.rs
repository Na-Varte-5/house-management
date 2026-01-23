use yew::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq)]
pub struct MeterReading {
    pub id: u64,
    pub meter_id: u64,
    pub reading_value: String,
    pub reading_timestamp: String,
    pub unit: String,
    pub source: String,
    pub created_at: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct ReadingHistoryProps {
    pub meter_id: u64,
    pub readings: Vec<MeterReading>,
    pub on_export: Callback<()>,
}

/// Component for displaying meter reading history table with CSV export
#[function_component(ReadingHistory)]
pub fn reading_history(props: &ReadingHistoryProps) -> Html {
    html! {
        <div class="card">
            <div class="card-header d-flex justify-content-between align-items-center">
                <h5 class="mb-0">{"Reading History"}</h5>
                <button class="btn btn-outline-primary btn-sm" onclick={props.on_export.reform(|_| ())}>
                    <i class="bi bi-download"></i> {"Export CSV"}
                </button>
            </div>
            <div class="card-body">
                if props.readings.is_empty() {
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
                                { for props.readings.iter().map(|reading| html! {
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
    }
}
