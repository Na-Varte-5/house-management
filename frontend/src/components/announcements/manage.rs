use super::{
    ActiveAnnouncementsList, AnnouncementItem, DeletedAnnouncement, DeletedAnnouncementsList,
};
use crate::components::announcement_editor::AnnouncementEditor;
use crate::components::announcement_editor::AnnouncementFull;
use crate::components::comment_list::CommentList;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::services::api_client;
use yew::prelude::*;

#[function_component(AnnouncementsManage)]
pub fn announcements_manage() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let token = auth.token().map(|t| t.to_string());

    let loading = use_state(|| false);
    let list = use_state(|| Vec::<AnnouncementItem>::new());
    let deleted = use_state(|| Vec::<DeletedAnnouncement>::new());
    let show_deleted = use_state(|| false);
    let selected = use_state(|| None::<u64>);
    let error = use_state(|| None::<String>);
    let editing = use_state(|| None::<AnnouncementItem>);
    let creating_new = use_state(|| false);

    let fetch_active = {
        let list = list.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();
        Callback::from(move |_| {
            loading.set(true);
            let list2 = list.clone();
            let loading2 = loading.clone();
            let error2 = error.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<serde_json::Value>>("/announcements").await {
                    Ok(v) => {
                        let mapped = v
                            .into_iter()
                            .filter_map(|x| {
                                Some(AnnouncementItem {
                                    id: x.get("id")?.as_u64()?,
                                    title: x.get("title")?.as_str()?.to_string(),
                                    body_html: x.get("body_html")?.as_str()?.to_string(),
                                    body_md: x.get("body_md")?.as_str()?.to_string(),
                                    author_id: x.get("author_id")?.as_u64()?,
                                    author_name: x.get("author_name")?.as_str()?.to_string(),
                                    pinned: x.get("pinned")?.as_bool()?,
                                    public: x.get("public")?.as_bool()?,
                                    roles_csv: x
                                        .get("roles_csv")
                                        .and_then(|r| r.as_str())
                                        .map(|s| s.to_string()),
                                    comments_enabled: x.get("comments_enabled")?.as_bool()?,
                                    publish_at: x
                                        .get("publish_at")
                                        .and_then(|r| r.as_str())
                                        .map(|s| s.to_string()),
                                    expire_at: x
                                        .get("expire_at")
                                        .and_then(|r| r.as_str())
                                        .map(|s| s.to_string()),
                                    is_deleted: x.get("is_deleted")?.as_bool()?,
                                    building_id: x.get("building_id").and_then(|r| r.as_u64()),
                                    building_address: x
                                        .get("building_address")
                                        .and_then(|r| r.as_str())
                                        .map(|s| s.to_string()),
                                    apartment_id: x.get("apartment_id").and_then(|r| r.as_u64()),
                                    apartment_number: x
                                        .get("apartment_number")
                                        .and_then(|r| r.as_str())
                                        .map(|s| s.to_string()),
                                })
                            })
                            .collect();
                        list2.set(mapped);
                        loading2.set(false);
                    }
                    Err(e) => {
                        error2.set(Some(format!("Load failed: {}", e)));
                        loading2.set(false);
                    }
                }
            });
        })
    };

    let fetch_deleted = {
        let deleted_state = deleted.clone();
        let error = error.clone();
        let show_deleted_flag = show_deleted.clone();
        let token = token.clone();
        Callback::from(move |_| {
            if !*show_deleted_flag {
                return;
            }
            let deleted_state2 = deleted_state.clone();
            let error2 = error.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<Vec<serde_json::Value>>("/announcements/deleted")
                    .await
                {
                    Ok(v) => {
                        let mapped = v
                            .into_iter()
                            .filter_map(|x| {
                                Some(DeletedAnnouncement {
                                    id: x.get("id")?.as_u64()?,
                                    title: x.get("title")?.as_str()?.to_string(),
                                })
                            })
                            .collect();
                        deleted_state2.set(mapped);
                    }
                    Err(e) => {
                        error2.set(Some(format!("Load deleted failed: {}", e)));
                    }
                }
            });
        })
    };

    // Initial load
    {
        let fetch_active = fetch_active.clone();
        use_effect_with((), move |_| {
            fetch_active.emit(());
            || ()
        });
    }

    // Reload deleted when toggled
    {
        let fetch_deleted = fetch_deleted.clone();
        use_effect_with(show_deleted.clone(), move |_| {
            fetch_deleted.emit(());
            || ()
        });
    }

    // Action handlers
    let publish_now = {
        let fetch_active = fetch_active.clone();
        let error = error.clone();
        let token = token.clone();
        Callback::from(move |id: u64| {
            let fetch_active2 = fetch_active.clone();
            let error2 = error.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .post_empty::<serde_json::Value>(&format!("/announcements/{}/publish", id))
                    .await
                {
                    Ok(_) => fetch_active2.emit(()),
                    Err(e) => error2.set(Some(format!("Publish failed: {}", e))),
                }
            });
        })
    };

    let toggle_pin = {
        let fetch_active = fetch_active.clone();
        let token = token.clone();
        Callback::from(move |id: u64| {
            let fetch_active2 = fetch_active.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(_) = client
                    .post_empty::<serde_json::Value>(&format!("/announcements/{}/pin", id))
                    .await
                {
                    fetch_active2.emit(());
                }
            });
        })
    };

    let soft_delete = {
        let fetch_active = fetch_active.clone();
        let token = token.clone();
        Callback::from(move |id: u64| {
            let fetch_active2 = fetch_active.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(_) = client
                    .delete_no_response(&format!("/announcements/{}", id))
                    .await
                {
                    fetch_active2.emit(());
                }
            });
        })
    };

    let restore = {
        let fetch_active = fetch_active.clone();
        let fetch_deleted = fetch_deleted.clone();
        let token = token.clone();
        Callback::from(move |id: u64| {
            let fetch_active2 = fetch_active.clone();
            let fetch_deleted2 = fetch_deleted.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(_) = client
                    .post_empty::<serde_json::Value>(&format!("/announcements/{}/restore", id))
                    .await
                {
                    fetch_active2.emit(());
                    fetch_deleted2.emit(());
                }
            });
        })
    };

    let purge = {
        let fetch_deleted = fetch_deleted.clone();
        let deleted_state = deleted.clone();
        let token = token.clone();
        Callback::from(move |id: u64| {
            let fetch_deleted2 = fetch_deleted.clone();
            let deleted_state2 = deleted_state.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(_) = client
                    .delete_no_response(&format!("/announcements/{}/purge", id))
                    .await
                {
                    deleted_state2.set(
                        deleted_state2
                            .iter()
                            .cloned()
                            .filter(|a| a.id != id)
                            .collect(),
                    );
                    fetch_deleted2.emit(());
                }
            });
        })
    };

    let toggle_comments = {
        let fetch_active = fetch_active.clone();
        let error = error.clone();
        let token = token.clone();
        Callback::from(move |(id, current): (u64, bool)| {
            let fetch_active2 = fetch_active.clone();
            let error2 = error.clone();
            let token = token.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({"comments_enabled": !current});
                let client = api_client(token.as_deref());
                match client
                    .put::<_, serde_json::Value>(&format!("/announcements/{}", id), &body)
                    .await
                {
                    Ok(_) => fetch_active2.emit(()),
                    Err(e) => error2.set(Some(format!("Toggle comments failed: {}", e))),
                }
            });
        })
    };

    // Editor management
    let on_created = {
        let fetch_active = fetch_active.clone();
        let editing = editing.clone();
        let creating_new = creating_new.clone();
        Callback::from(move |_: AnnouncementFull| {
            fetch_active.emit(());
            editing.set(None);
            creating_new.set(false);
        })
    };

    let on_updated = {
        let fetch_active = fetch_active.clone();
        let editing = editing.clone();
        Callback::from(move |_: AnnouncementFull| {
            fetch_active.emit(());
            editing.set(None);
        })
    };

    let on_published = {
        let fetch_active = fetch_active.clone();
        let editing = editing.clone();
        Callback::from(move |_: AnnouncementFull| {
            fetch_active.emit(());
            editing.set(None);
        })
    };

    let start_new = {
        let creating_new = creating_new.clone();
        let editing = editing.clone();
        Callback::from(move |_e: web_sys::MouseEvent| {
            editing.set(None);
            creating_new.set(true);
        })
    };

    let cancel_new = {
        let creating_new = creating_new.clone();
        Callback::from(move |_| creating_new.set(false))
    };

    let cancel_edit = {
        let editing = editing.clone();
        Callback::from(move |_| editing.set(None))
    };

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |id: u64| selected.set(Some(id)))
    };

    let on_edit = {
        let editing = editing.clone();
        Callback::from(move |item: AnnouncementItem| editing.set(Some(item)))
    };

    let selected_item = selected.and_then(|id| list.iter().find(|a| a.id == id).cloned());
    let now_iso: String = js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default();

    // Convert editing item to full format
    let editing_full: Option<AnnouncementFull> = (*editing).clone().map(|a| AnnouncementFull {
        id: a.id,
        title: a.title,
        body_md: a.body_md,
        body_html: a.body_html,
        pinned: a.pinned,
        public: a.public,
        roles_csv: a.roles_csv,
        building_id: a.building_id,
        apartment_id: a.apartment_id,
        comments_enabled: a.comments_enabled,
        publish_at: a.publish_at,
        expire_at: a.expire_at,
    });

    html! {
        <div class="announcements-manage mb-4">
            { if let Some(err) = &*error {
                html!{<div class="alert alert-danger py-1 mb-2">{err}</div>}
            } else {
                html!{}
            } }

            <div class="d-flex justify-content-between align-items-center mb-3">
                <div>
                    <h4 class="mb-0">{ t("announcements-title") }</h4>
                    <div class="text-muted small">{ t("announcements-manage-subtitle") }</div>
                </div>
                <div class="d-flex align-items-center gap-2">
                    <button class="btn btn-sm btn-primary" onclick={start_new.clone()}>
                        { t("announcement-new-button") }
                    </button>
                    <div class="form-check form-switch mb-0">
                        <input
                            class="form-check-input"
                            type="checkbox"
                            id="showDeletedAnnouncements"
                            checked={*show_deleted}
                            onchange={{
                                let sd = show_deleted.clone();
                                Callback::from(move |e: Event| {
                                    let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    sd.set(i.checked());
                                })
                            }}
                        />
                        <label class="form-check-label small" for="showDeletedAnnouncements">
                            { t("announcement-show-deleted-toggle") }
                        </label>
                    </div>
                </div>
            </div>

            // Create new editor
            { if *creating_new {
                html! {
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{"New announcement"}</span>
                            <button
                                class="btn btn-sm btn-outline-secondary"
                                onclick={Callback::from({
                                    let cancel = cancel_new.clone();
                                    move |_e: web_sys::MouseEvent| cancel.emit(())
                                })}
                            >
                                {"Cancel"}
                            </button>
                        </div>
                        <div class="card-body">
                            <AnnouncementEditor on_created={on_created.clone()} on_cancel={cancel_new} />
                        </div>
                    </div>
                }
            } else {
                html!{}
            } }

            // Edit editor
            { if let Some(full) = editing_full {
                html! {
                    <div class="card mb-3">
                        <div class="card-header d-flex justify-content-between align-items-center">
                            <span>{"Edit announcement"}</span>
                            <button
                                class="btn btn-sm btn-outline-secondary"
                                onclick={Callback::from({
                                    let cancel = cancel_edit.clone();
                                    move |_e: web_sys::MouseEvent| cancel.emit(())
                                })}
                            >
                                {"Cancel"}
                            </button>
                        </div>
                        <div class="card-body">
                            <AnnouncementEditor
                                existing={Some(full)}
                                on_updated={on_updated.clone()}
                                on_published={on_published.clone()}
                                on_cancel={cancel_edit.clone()}
                            />
                        </div>
                    </div>
                }
            } else {
                html!{}
            } }

            // Active announcements list component
            <ActiveAnnouncementsList
                announcements={(*list).clone()}
                loading={*loading}
                on_select={on_select}
                on_publish_now={publish_now}
                on_edit={on_edit}
                on_toggle_pin={toggle_pin}
                on_toggle_comments={toggle_comments.clone()}
                on_delete={soft_delete}
            />

            // Deleted announcements list component
            { if *show_deleted {
                html!{
                    <DeletedAnnouncementsList
                        announcements={(*deleted).clone()}
                        on_restore={restore}
                        on_purge={purge}
                    />
                }
            } else {
                html!{}
            } }

            // Selected announcement detail view
            { if let Some(item) = selected_item {
                html!{
                    <div class="mt-3">
                        { {
                            let mut nodes: Vec<Html> = Vec::new();
                            if item.pinned {
                                nodes.push(html!{<span class="badge bg-warning text-dark me-1">{"Pinned"}</span>});
                            }
                            if let Some(p) = &item.publish_at {
                                if p > &now_iso {
                                    nodes.push(html!{<span class="badge bg-info text-dark me-1">{"Scheduled"}</span>});
                                }
                            }
                            if let Some(e) = &item.expire_at {
                                if e < &now_iso {
                                    nodes.push(html!{<span class="badge bg-dark me-1">{"Expired"}</span>});
                                }
                            }
                            html!{<>{ for nodes }</>}
                        }}
                        <h5 class="fw-bold mb-1">{item.title.clone()}</h5>
                        <div class="small text-muted mb-2">{format!("By {}", item.author_name)}</div>
                        { {
                            let mut badges: Vec<Html> = Vec::new();
                            if item.public {
                                badges.push(html!{<span class="badge bg-success me-1">{"Public"}</span>});
                            } else {
                                badges.push(html!{<span class="badge bg-secondary me-1">{"Private"}</span>});
                            }
                            if let Some(csv) = &item.roles_csv {
                                for role in csv.split(',').map(|r| r.trim()).filter(|r| !r.is_empty()) {
                                    badges.push(html!{<span class="badge bg-primary me-1">{role}</span>});
                                }
                            }
                            if let Some(addr) = item.building_address.clone() {
                                badges.push(html!{<span class="badge bg-info text-dark me-1">{format!("Bldg: {}", addr)}</span>});
                            }
                            if let Some(num) = item.apartment_number.clone() {
                                badges.push(html!{<span class="badge bg-warning text-dark me-1">{format!("Apt: {}", num)}</span>});
                            }
                            html!{<div class="mb-2">{ for badges }</div>}
                        }}
                        <div class="border rounded p-2">
                            { Html::from_html_unchecked(item.body_html.into()) }
                        </div>
                        <div class="mb-2">
                            <button
                                class="btn btn-sm btn-outline-warning"
                                onclick={Callback::from(move |_| toggle_comments.emit((item.id, item.comments_enabled)))}
                            >
                                { if item.comments_enabled { "Disable Comments" } else { "Enable Comments" } }
                            </button>
                        </div>
                        <CommentList announcement_id={item.id} comments_enabled={item.comments_enabled} />
                    </div>
                }
            } else {
                html!{}
            } }
        </div>
    }
}
