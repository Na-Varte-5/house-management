use yew::prelude::*;
use crate::components::spinner::Spinner;

#[derive(Clone, PartialEq, Debug)]
pub struct AnnouncementItem {
    pub id: u64,
    pub title: String,
    pub body_html: String,
    pub body_md: String,
    pub author_id: u64,
    pub author_name: String,
    pub pinned: bool,
    pub public: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub building_address: Option<String>,
    pub apartment_id: Option<u64>,
    pub apartment_number: Option<String>,
    pub comments_enabled: bool,
    pub publish_at: Option<String>,
    pub expire_at: Option<String>,
    pub is_deleted: bool,
}

#[derive(Properties, PartialEq)]
pub struct ActiveAnnouncementsListProps {
    pub announcements: Vec<AnnouncementItem>,
    pub loading: bool,
    pub on_select: Callback<u64>,
    pub on_publish_now: Callback<u64>,
    pub on_edit: Callback<AnnouncementItem>,
    pub on_toggle_pin: Callback<u64>,
    pub on_toggle_comments: Callback<(u64, bool)>,
    pub on_delete: Callback<u64>,
}

/// Component for displaying list of active announcements with action buttons
#[function_component(ActiveAnnouncementsList)]
pub fn active_announcements_list(props: &ActiveAnnouncementsListProps) -> Html {
    let now_iso: String = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();

    html! {
        <div>
            <div class="d-flex justify-content-between align-items-center mb-2">
                <h6 class="mb-0">{"Active"}</h6>
            </div>
            { if props.loading {
                html!{<Spinner center={true} />}
            } else {
                html!{
                    <div>
                        { for props.announcements.iter().cloned().map(|a| {
                            let item_id = a.id;
                            let item_pinned = a.pinned;
                            let item_title = a.title.clone();
                            let item_publish_at = a.publish_at.clone();
                            let item_public = a.public;
                            let item_roles = a.roles_csv.clone();
                            let item_comments = a.comments_enabled;
                            let item_expire_at = a.expire_at.clone();
                            let item_author_name = a.author_name.clone();

                            // Status badges
                            let status_badges: Html = {
                                let mut nodes: Vec<Html> = Vec::new();
                                if item_pinned {
                                    nodes.push(html!{<span class="badge bg-warning text-dark me-1">{"Pinned"}</span>});
                                }
                                if let Some(p) = &item_publish_at {
                                    if p > &now_iso {
                                        nodes.push(html!{<span class="badge bg-info text-dark me-1">{"Scheduled"}</span>});
                                    }
                                }
                                if let Some(e) = &item_expire_at {
                                    if e < &now_iso {
                                        nodes.push(html!{<span class="badge bg-dark me-1">{"Expired"}</span>});
                                    }
                                }
                                html!{<>{ for nodes }</>}
                            };

                            // Scope badges
                            let scope_badges: Html = {
                                let mut badges: Vec<Html> = Vec::new();
                                if item_public {
                                    badges.push(html!{<span class="badge bg-success me-1">{"Public"}</span>});
                                } else {
                                    badges.push(html!{<span class="badge bg-secondary me-1">{"Private"}</span>});
                                }
                                if let Some(csv) = &item_roles {
                                    for role in csv.split(',').map(|r| r.trim()).filter(|r| !r.is_empty()) {
                                        badges.push(html!{<span class="badge bg-primary me-1">{role}</span>});
                                    }
                                }
                                if let Some(addr) = a.building_address.clone() {
                                    badges.push(html!{<span class="badge bg-info text-dark me-1">{format!("Bldg: {}", addr)}</span>});
                                }
                                if let Some(num) = a.apartment_number.clone() {
                                    badges.push(html!{<span class="badge bg-warning text-dark me-1">{format!("Apt: {}", num)}</span>});
                                }
                                html!{<div class="mt-1">{ for badges }</div>}
                            };

                            let on_select = props.on_select.clone();
                            let on_publish = props.on_publish_now.clone();
                            let on_edit = props.on_edit.clone();
                            let on_pin = props.on_toggle_pin.clone();
                            let on_comments = props.on_toggle_comments.clone();
                            let on_del = props.on_delete.clone();
                            let item_for_edit = a.clone();

                            html!{
                                <div class="border rounded p-2 mb-2" key={item_id}>
                                    <div class="d-flex justify-content-between align-items-center">
                                        <div style="cursor:pointer" onclick={Callback::from(move |_| on_select.emit(item_id))}>
                                            {status_badges}
                                            <h6 class="d-inline fw-bold mb-0 ms-1">{item_title.clone()}</h6>
                                            { if item_publish_at.is_none() {
                                                html!{<span class="badge bg-secondary ms-1">{"Draft"}</span>}
                                            } else {
                                                html!{}
                                            } }
                                            <div class="small text-muted">{format!("By {}", item_author_name)}</div>
                                            {scope_badges}
                                        </div>
                                        <div class="btn-group btn-group-sm">
                                            { if item_publish_at.is_none() {
                                                html!{
                                                    <button
                                                        class="btn btn-outline-success"
                                                        onclick={Callback::from(move |_| on_publish.emit(item_id))}
                                                    >
                                                        {"Publish Now"}
                                                    </button>
                                                }
                                            } else {
                                                html!{}
                                            } }
                                            <button
                                                class="btn btn-outline-primary"
                                                onclick={Callback::from({
                                                    let item = item_for_edit.clone();
                                                    move |_| on_edit.emit(item.clone())
                                                })}
                                            >
                                                {"Edit"}
                                            </button>
                                            <button
                                                class="btn btn-outline-secondary"
                                                onclick={Callback::from(move |_| on_pin.emit(item_id))}
                                            >
                                                {"Pin"}
                                            </button>
                                            <button
                                                class="btn btn-outline-warning"
                                                onclick={Callback::from(move |_| on_comments.emit((item_id, item_comments)))}
                                            >
                                                { if item_comments { "Disable Comments" } else { "Enable Comments" } }
                                            </button>
                                            <button
                                                class="btn btn-outline-danger"
                                                onclick={Callback::from(move |_| on_del.emit(item_id))}
                                            >
                                                {"Delete"}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }) }
                    </div>
                }
            } }
        </div>
    }
}
