use crate::components::announcement_editor_form::AnnouncementEditorForm;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::services::api_client;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize)]
pub struct AnnouncementFull {
    pub id: u64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub pinned: bool,
    pub public: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub apartment_id: Option<u64>,
    pub comments_enabled: bool,
    pub publish_at: Option<String>,
    pub expire_at: Option<String>,
}

#[derive(Serialize)]
struct CreatePayload {
    title: String,
    body_md: String,
    public: bool,
    pinned: bool,
    roles_csv: Option<String>,
    building_id: Option<u64>,
    apartment_id: Option<u64>,
    comments_enabled: bool,
    publish_at: Option<String>,
    expire_at: Option<String>,
}

#[derive(Deserialize)]
struct BuildingDto {
    id: u64,
    address: String,
}

#[derive(Deserialize)]
struct ApartmentDto {
    id: u64,
    building_id: u64,
    number: String,
}

#[derive(Properties, PartialEq)]
pub struct EditorProps {
    #[prop_or_default]
    pub on_created: Callback<AnnouncementFull>,
    #[prop_or_default]
    pub on_updated: Callback<AnnouncementFull>,
    #[prop_or_default]
    pub on_published: Callback<AnnouncementFull>,
    #[prop_or_default]
    pub existing: Option<AnnouncementFull>,
    #[prop_or_default]
    pub on_cancel: Callback<()>,
}

/// Orchestrator component that manages state and API calls for announcement editing
#[function_component(AnnouncementEditor)]
pub fn announcement_editor(props: &EditorProps) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let token = auth.token().map(|t| t.to_string());

    // Form state
    let title = use_state(String::default);
    let body_md = use_state(String::default);
    let public_flag = use_state(|| true);
    let pinned_flag = use_state(|| false);
    let comments_enabled = use_state(|| false);
    let publish_at = use_state(String::default);
    let expire_at = use_state(String::default);
    let selected_roles = use_state(|| Vec::<String>::new());
    let selected_building = use_state(|| None::<u64>);
    let selected_apartment = use_state(|| None::<u64>);

    // Data state
    let buildings = use_state(|| Vec::<(u64, String)>::new());
    let apartments = use_state(|| Vec::<(u64, u64, String)>::new());

    // UI state
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);

    // Memoized preview HTML
    let preview_html = {
        let md = (*body_md).clone();
        use_memo(md.clone(), move |current_md| {
            if current_md.is_empty() {
                return format!("<em class='text-muted'>{}</em>", t("preview-empty"));
            }
            let mut buf = String::new();
            let parser =
                pulldown_cmark::Parser::new_ext(current_md, pulldown_cmark::Options::all());
            pulldown_cmark::html::push_html(&mut buf, parser);
            let sanitized = ammonia::Builder::default().clean(&buf).to_string();
            if sanitized.trim().is_empty() {
                format!("<pre>{}</pre>", html_escape::encode_text(current_md))
            } else {
                sanitized
            }
        })
    };

    // Initialize state from existing announcement
    {
        let title_state = title.clone();
        let body_state = body_md.clone();
        let public_state = public_flag.clone();
        let pinned_state = pinned_flag.clone();
        let comments_state = comments_enabled.clone();
        let publish_state = publish_at.clone();
        let expire_state = expire_at.clone();
        let selected_roles_state = selected_roles.clone();
        let selected_building_state = selected_building.clone();
        let selected_apartment_state = selected_apartment.clone();

        use_effect_with(props.existing.clone(), move |ex| {
            if let Some(a) = ex {
                title_state.set(a.title.clone());
                body_state.set(a.body_md.clone());
                public_state.set(a.public);
                pinned_state.set(a.pinned);
                comments_state.set(a.comments_enabled);
                publish_state.set(a.publish_at.clone().unwrap_or_default().trim().to_string());
                expire_state.set(a.expire_at.clone().unwrap_or_default().trim().to_string());
                selected_roles_state.set(
                    a.roles_csv
                        .clone()
                        .map(|csv| {
                            csv.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        })
                        .unwrap_or_default(),
                );
                selected_building_state.set(a.building_id);
                selected_apartment_state.set(a.apartment_id);
            } else {
                // Reset for new announcement
                title_state.set(String::new());
                body_state.set(String::new());
                public_state.set(true);
                pinned_state.set(false);
                comments_state.set(false);
                publish_state.set(String::new());
                expire_state.set(String::new());
                selected_roles_state.set(Vec::new());
                selected_building_state.set(None);
                selected_apartment_state.set(None);
            }
        });
    }

    // Load buildings
    {
        let buildings_state = buildings.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(None);
                if let Ok(list) = client.get::<Vec<BuildingDto>>("/buildings").await {
                    let mapped = list.into_iter().map(|b| (b.id, b.address)).collect();
                    buildings_state.set(mapped);
                }
            });
            || ()
        });
    }

    // Load apartments when building changes
    {
        let apartments_state = apartments.clone();
        let selected_building_state = selected_building.clone();
        use_effect_with(selected_building.clone(), move |_| {
            apartments_state.set(Vec::new());
            if let Some(bid) = *selected_building_state {
                let apartments_state2 = apartments_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(None);
                    let endpoint = format!("/buildings/{}/apartments", bid);
                    if let Ok(list) = client.get::<Vec<ApartmentDto>>(&endpoint).await {
                        let mapped = list
                            .into_iter()
                            .map(|a| (a.id, a.building_id, a.number))
                            .collect();
                        apartments_state2.set(mapped);
                    }
                });
            }
            || ()
        });
    }

    // Form callbacks
    let on_title_change = {
        let title = title.clone();
        Callback::from(move |value: String| title.set(value))
    };

    let on_body_md_change = {
        let body_md = body_md.clone();
        Callback::from(move |value: String| body_md.set(value))
    };

    let on_public_change = {
        let public_flag = public_flag.clone();
        Callback::from(move |value: bool| public_flag.set(value))
    };

    let on_pinned_change = {
        let pinned_flag = pinned_flag.clone();
        Callback::from(move |value: bool| pinned_flag.set(value))
    };

    let on_comments_change = {
        let comments_enabled = comments_enabled.clone();
        Callback::from(move |value: bool| comments_enabled.set(value))
    };

    let on_publish_at_change = {
        let publish_at = publish_at.clone();
        Callback::from(move |value: String| publish_at.set(value))
    };

    let on_expire_at_change = {
        let expire_at = expire_at.clone();
        Callback::from(move |value: String| expire_at.set(value))
    };

    let on_roles_change = {
        let selected_roles = selected_roles.clone();
        Callback::from(move |value: Vec<String>| selected_roles.set(value))
    };

    let on_building_change = {
        let selected_building = selected_building.clone();
        Callback::from(move |value: Option<u64>| selected_building.set(value))
    };

    let on_apartment_change = {
        let selected_apartment = selected_apartment.clone();
        Callback::from(move |value: Option<u64>| selected_apartment.set(value))
    };

    // Submit handler
    let on_submit = {
        let title = title.clone();
        let body_md = body_md.clone();
        let public_flag = public_flag.clone();
        let pinned_flag = pinned_flag.clone();
        let comments_enabled = comments_enabled.clone();
        let publish_at = publish_at.clone();
        let expire_at = expire_at.clone();
        let saving = saving.clone();
        let error = error.clone();
        let on_created = props.on_created.clone();
        let on_updated = props.on_updated.clone();
        let existing = props.existing.clone();
        let selected_roles = selected_roles.clone();
        let selected_building = selected_building.clone();
        let selected_apartment = selected_apartment.clone();
        let token = token.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if (*title).trim().is_empty() {
                error.set(Some(t("announcement-title-label")));
                return;
            }
            if (*body_md).trim().is_empty() {
                error.set(Some(t("announcement-body-label")));
                return;
            }
            error.set(None);
            saving.set(true);

            let client = api_client(token.as_deref());
            let title_val = (*title).clone();
            let body_md_val = (*body_md).clone();
            let public_val = *public_flag;
            let pinned_val = *pinned_flag;
            let comments_val = *comments_enabled;
            let publish_at_val = (*publish_at).clone();
            let expire_at_val = (*expire_at).clone();
            let roles_val = (*selected_roles).clone();
            let building_val = *selected_building;
            let apartment_val = *selected_apartment;

            if let Some(ex) = &existing {
                // Update existing
                let id = ex.id;
                let roles_string = if roles_val.is_empty() {
                    None
                } else {
                    Some(roles_val.join(","))
                };
                let payload = serde_json::json!({
                    "title": title_val,
                    "body_md": body_md_val,
                    "public": public_val,
                    "pinned": pinned_val,
                    "roles_csv": roles_string.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                    "comments_enabled": comments_val,
                    "publish_at": if publish_at_val.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(datetime_local_to_naive(&publish_at_val)) },
                    "expire_at": if expire_at_val.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(datetime_local_to_naive(&expire_at_val)) },
                    "building_id": building_val.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
                    "apartment_id": apartment_val.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null),
                });
                let saving2 = saving.clone();
                let error2 = error.clone();
                let on_updated_cb = on_updated.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let endpoint = format!("/announcements/{}", id);
                    match client
                        .put::<serde_json::Value, AnnouncementFull>(&endpoint, &payload)
                        .await
                    {
                        Ok(updated) => on_updated_cb.emit(updated),
                        Err(_) => error2.set(Some(t("error-load-failed"))),
                    }
                    saving2.set(false);
                });
            } else {
                // Create new
                let roles_csv_opt = if roles_val.is_empty() {
                    None
                } else {
                    Some(roles_val.join(","))
                };
                let payload = CreatePayload {
                    title: title_val,
                    body_md: body_md_val,
                    public: public_val,
                    pinned: pinned_val,
                    roles_csv: roles_csv_opt,
                    building_id: building_val,
                    apartment_id: apartment_val,
                    comments_enabled: comments_val,
                    publish_at: if publish_at_val.trim().is_empty() {
                        None
                    } else {
                        Some(datetime_local_to_naive(&publish_at_val))
                    },
                    expire_at: if expire_at_val.trim().is_empty() {
                        None
                    } else {
                        Some(datetime_local_to_naive(&expire_at_val))
                    },
                };
                let saving2 = saving.clone();
                let error2 = error.clone();
                let on_created_outer = on_created.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    match client
                        .post::<_, AnnouncementFull>("/announcements", &payload)
                        .await
                    {
                        Ok(created) => on_created_outer.emit(created),
                        Err(e) => error2.set(Some(format!("{}: {}", t("error-post-failed"), e))),
                    }
                    saving2.set(false);
                });
            }
        })
    };

    // Publish now handler
    let publish_now_id = if let Some(ex) = &props.existing {
        if ex.publish_at.is_none() {
            Some(ex.id)
        } else {
            None
        }
    } else {
        None
    };

    let on_publish_now = {
        let on_published = props.on_published.clone();
        let saving = saving.clone();
        let error = error.clone();
        let token = token.clone();

        Callback::from(move |id: u64| {
            saving.set(true);
            let saving2 = saving.clone();
            let error2 = error.clone();
            let on_published_cb = on_published.clone();
            let client = api_client(token.as_deref());

            wasm_bindgen_futures::spawn_local(async move {
                let endpoint = format!("/announcements/{}/publish", id);
                match client.post_empty::<AnnouncementFull>(&endpoint).await {
                    Ok(updated) => on_published_cb.emit(updated),
                    Err(_) => error2.set(Some(t("error-post-failed"))),
                }
                saving2.set(false);
            });
        })
    };

    html! {
        <AnnouncementEditorForm
            title={(*title).clone()}
            body_md={(*body_md).clone()}
            public_flag={*public_flag}
            pinned_flag={*pinned_flag}
            comments_enabled={*comments_enabled}
            publish_at={(*publish_at).clone()}
            expire_at={(*expire_at).clone()}
            selected_roles={(*selected_roles).clone()}
            selected_building={*selected_building}
            selected_apartment={*selected_apartment}
            buildings={(*buildings).clone()}
            apartments={(*apartments).clone()}
            preview_html={(*preview_html).clone()}
            saving={*saving}
            error={(*error).clone()}
            is_editing={props.existing.is_some()}
            publish_now_id={publish_now_id}
            on_title_change={on_title_change}
            on_body_md_change={on_body_md_change}
            on_public_change={on_public_change}
            on_pinned_change={on_pinned_change}
            on_comments_change={on_comments_change}
            on_publish_at_change={on_publish_at_change}
            on_expire_at_change={on_expire_at_change}
            on_roles_change={on_roles_change}
            on_building_change={on_building_change}
            on_apartment_change={on_apartment_change}
            on_submit={on_submit}
            on_publish_now={on_publish_now}
            on_cancel={props.on_cancel.clone()}
        />
    }
}

fn datetime_local_to_naive(val: &str) -> String {
    // HTML datetime-local gives YYYY-MM-DDTHH:MM; append :00 seconds if missing
    if val.len() == 16 {
        format!("{}:00", val)
    } else {
        val.to_string()
    }
}
