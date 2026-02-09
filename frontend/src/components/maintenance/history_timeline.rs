use crate::i18n::t;
use crate::utils::datetime::format_dt_local;
use serde::Deserialize;
use yew::prelude::*;

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

#[function_component(HistoryTimeline)]
pub fn history_timeline(props: &HistoryTimelineProps) -> Html {
    html! {
        <div class="card mt-3">
            <div class="card-header">
                <h5 class="mb-0">{t("maintenance-history")}</h5>
            </div>
            <div class="card-body">
                if props.loading {
                    <div class="text-center">
                        <div class="spinner-border spinner-border-sm" role="status"></div>
                    </div>
                } else if props.history.is_empty() {
                    <p class="text-muted small mb-0">{t("maintenance-no-history")}</p>
                } else {
                    <div class="timeline">
                        {
                            for props.history.iter().map(|entry| {
                                let formatted_date = entry.changed_at.as_ref()
                                    .map(|dt| format_dt_local(dt))
                                    .unwrap_or_else(|| "N/A".to_string());

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
                                                            {t("maintenance-changed-status")}
                                                            {" "}
                                                            <span class="text-decoration-line-through">
                                                                {entry.from_status.as_ref().unwrap_or(&t("maintenance-status-none"))}
                                                            </span>
                                                            {" "}
                                                            {t("maintenance-status-to")}
                                                            {" "}
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
