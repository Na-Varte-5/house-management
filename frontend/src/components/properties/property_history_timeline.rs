use crate::components::ErrorAlert;
use crate::contexts::auth::AuthContext;
use crate::services::api::{ApiError, api_client};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyHistoryEvent {
    pub id: u64,
    pub apartment_id: u64,
    pub event_type: String,
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub changed_by: u64,
    pub changed_by_name: String,
    pub description: String,
    pub metadata: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub apartment_id: u64,
}

#[function_component(PropertyHistoryTimeline)]
pub fn property_history_timeline(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let events = use_state(|| Vec::<PropertyHistoryEvent>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    // Fetch history on mount
    {
        let events = events.clone();
        let loading = loading.clone();
        let error = error.clone();
        let apartment_id = props.apartment_id;
        let token = auth.token().map(|t| t.to_string());

        use_effect_with(apartment_id, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);

                match fetch_property_history(apartment_id, token.as_deref()).await {
                    Ok(history) => {
                        events.set(history);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load property history: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="property-history-timeline">
            <h5 class="mb-3">{"Property History"}</h5>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} />
            }

            if *loading {
                <div class="text-center py-4">
                    <div class="spinner-border text-primary" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if events.is_empty() {
                <div class="alert alert-info">
                    <i class="bi bi-info-circle me-2"></i>
                    {"No property history events yet."}
                </div>
            } else {
                <div class="timeline">
                    { for events.iter().map(|event| render_event(event)) }
                </div>
            }
        </div>
    }
}

fn render_event(event: &PropertyHistoryEvent) -> Html {
    let icon_class = match event.event_type.as_str() {
        "owner_added" => "bi-person-plus-fill text-success",
        "owner_removed" => "bi-person-dash-fill text-danger",
        "renter_added" => "bi-house-add-fill text-info",
        "renter_updated" => "bi-pencil-fill text-warning",
        "renter_removed" => "bi-house-dash-fill text-secondary",
        _ => "bi-circle-fill text-muted",
    };

    let formatted_date = if let Some(date_str) = &event.created_at {
        format_datetime(date_str)
    } else {
        "Unknown date".to_string()
    };

    html! {
        <div class="timeline-item mb-3">
            <div class="d-flex">
                <div class="timeline-marker me-3">
                    <i class={format!("bi {} fs-4", icon_class)}></i>
                </div>
                <div class="timeline-content flex-grow-1">
                    <div class="card">
                        <div class="card-body">
                            <h6 class="card-title mb-1">
                                {&event.description}
                            </h6>
                            <p class="card-text text-muted small mb-2">
                                <i class="bi bi-clock me-1"></i>
                                {formatted_date}
                                {" by "}
                                <strong>{&event.changed_by_name}</strong>
                            </p>
                            if let Some(metadata) = &event.metadata {
                                if !metadata.is_empty() {
                                    <div class="small text-muted">
                                        {render_metadata(metadata)}
                                    </div>
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn render_metadata(metadata_json: &str) -> Html {
    // Try to parse and pretty-print the metadata
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(metadata_json) {
        if let Some(obj) = parsed.as_object() {
            return html! {
                <ul class="list-unstyled mb-0 mt-1">
                    { for obj.iter().map(|(key, value)| {
                        let formatted_value = match value {
                            serde_json::Value::String(s) => {
                                if s == "null" {
                                    "Not set".to_string()
                                } else {
                                    s.clone()
                                }
                            },
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => "Not set".to_string(),
                            _ => value.to_string().trim_matches('"').to_string(),
                        };

                        html! {
                            <li key={key.clone()}>
                                <strong>{format!("{}:", key.replace('_', " "))}</strong>
                                {" "}
                                {formatted_value}
                            </li>
                        }
                    }) }
                </ul>
            };
        }
    }

    html! {
        <code class="small">{metadata_json}</code>
    }
}

fn format_datetime(date_str: &str) -> String {
    // Parse ISO datetime and format it nicely
    // Input format: "2026-01-24T14:30:00"
    // Output format: "Jan 24, 2026 at 14:30"

    let parts: Vec<&str> = date_str.split('T').collect();
    if parts.len() != 2 {
        return date_str.to_string();
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return date_str.to_string();
    }

    let month = match date_parts[1] {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => return date_str.to_string(),
    };

    format!(
        "{} {}, {} at {}:{}",
        month,
        date_parts[2].trim_start_matches('0'),
        date_parts[0],
        time_parts[0],
        time_parts[1]
    )
}

async fn fetch_property_history(
    apartment_id: u64,
    token: Option<&str>,
) -> Result<Vec<PropertyHistoryEvent>, ApiError> {
    let client = api_client(token);
    let url = format!("/api/v1/apartments/{}/history", apartment_id);
    client.get(&url).await
}
