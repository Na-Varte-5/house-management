use yew::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Clone, PartialEq)]
pub struct HistoryEntry {
    pub id: u64,
    pub request_id: u64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub changed_by: u64,
    pub changed_by_name: String,
    pub changed_at: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct HistoryTimelineProps {
    pub history: Vec<HistoryEntry>,
    pub loading: bool,
}

/// Helper function to format datetime strings to be more user-friendly
fn format_date(datetime_str: &str) -> String {
    if datetime_str.is_empty() {
        return String::from("N/A");
    }

    // Parse the datetime string (format: "2026-01-14 10:30:00")
    let parts: Vec<&str> = datetime_str.split(' ').collect();
    if parts.len() == 2 {
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        if date_parts.len() == 3 {
            let year = date_parts[0];
            let month = date_parts[1];
            let day = date_parts[2];
            let time = parts[1];

            // Format as "Jan 14, 2026 at 10:30"
            let month_name = match month {
                "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
                "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
                "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
                _ => month
            };

            let time_parts: Vec<&str> = time.split(':').collect();
            let short_time = if time_parts.len() >= 2 {
                format!("{}:{}", time_parts[0], time_parts[1])
            } else {
                time.to_string()
            };

            return format!("{} {}, {} at {}", month_name, day, year, short_time);
        }
    }

    datetime_str.to_string()
}

/// Timeline component for displaying maintenance request history
#[function_component(HistoryTimeline)]
pub fn history_timeline(props: &HistoryTimelineProps) -> Html {
    html! {
        <div class="card mt-3">
            <div class="card-header">
                <h5 class="mb-0">{"History"}</h5>
            </div>
            <div class="card-body">
                if props.loading {
                    <div class="text-center">
                        <div class="spinner-border spinner-border-sm" role="status"></div>
                    </div>
                } else if props.history.is_empty() {
                    <p class="text-muted small mb-0">{"No history available"}</p>
                } else {
                    <div class="timeline">
                        {
                            for props.history.iter().map(|entry| {
                                let formatted_date = entry.changed_at.as_ref()
                                    .map(|dt| format_date(dt))
                                    .unwrap_or_else(|| "(unknown)".to_string());

                                html! {
                                    <div class="mb-3 pb-3 border-bottom">
                                        <div class="d-flex justify-content-between">
                                            <strong class="small">{&entry.changed_by_name}</strong>
                                            <span class="small text-muted">
                                                {formatted_date}
                                            </span>
                                        </div>
                                        <p class="mb-0 small">
                                            {
                                                // Check if this is a status change or other type of change
                                                if entry.from_status.is_none() && entry.note.is_some() {
                                                    // Priority or assignment change - just show the note
                                                    html! { <>{entry.note.as_ref().unwrap()}</> }
                                                } else {
                                                    // Status change - show from/to format
                                                    html! {
                                                        <>
                                                            {"Changed status from "}
                                                            <span class="text-decoration-line-through">
                                                                {entry.from_status.as_ref().unwrap_or(&"(none)".to_string())}
                                                            </span>
                                                            {" to "}
                                                            <strong>{&entry.to_status}</strong>
                                                            {
                                                                if let Some(note) = &entry.note {
                                                                    html! { <span class="text-muted">{" - "}{note}</span> }
                                                                } else {
                                                                    html! {}
                                                                }
                                                            }
                                                        </>
                                                    }
                                                }
                                            }
                                        </p>
                                    </div>
                                }
                            })
                        }
                    </div>
                }
            </div>
        </div>
    }
}
