use crate::components::spinner::Spinner;
use crate::i18n::t;
use crate::utils::{api::api_url, auth::get_token};
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
    pub on_cancel: Callback<()> ,
}

#[function_component(AnnouncementEditor)]
pub fn announcement_editor(props: &EditorProps) -> Html {
    let title = use_state(String::default);
    let body_md = use_state(String::default);
    let public_flag = use_state(|| true);
    let pinned_flag = use_state(|| false);
    let comments_enabled = use_state(|| false);
    let publish_at = use_state(String::default); // datetime-local value
    let expire_at = use_state(String::default);
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);
    let active_tab = use_state(|| "edit".to_string());
    let selected_roles = use_state(|| Vec::<String>::new());
    let buildings = use_state(|| Vec::<(u64,String)>::new());
    let apartments = use_state(|| Vec::<(u64,u64,String)>::new()); // (id, building_id, number)
    let selected_building = use_state(|| None::<u64>);
    let selected_apartment = use_state(|| None::<u64>);

    // Memoized preview HTML (updates whenever body_md changes)
    let preview_html_computed = {
        let md = (*body_md).clone();
        use_memo(md.clone(), move |current_md| {
            if current_md.is_empty() {
                return format!("<em class='text-muted'>{}</em>", t("preview-empty"));
            }
            let mut buf = String::new();
            let parser = pulldown_cmark::Parser::new_ext(current_md, pulldown_cmark::Options::all());
            pulldown_cmark::html::push_html(&mut buf, parser);
            let sanitized = ammonia::Builder::default().clean(&buf).to_string();
            if sanitized.trim().is_empty() { format!("<pre>{}</pre>", html_escape::encode_text(current_md)) } else { sanitized }
        })
    };

    // Initialize state from existing announcement when switching to edit mode
    {
        let _existing = props.existing.clone();
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
                selected_roles_state.set(a.roles_csv.clone().map(|csv| csv.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default());
                selected_building_state.set(a.building_id);
                selected_apartment_state.set(a.apartment_id);
            } else {
                // reset if leaving edit mode
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

    // Load buildings and apartments for selectors
    {
        let buildings_state = buildings.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = reqwasm::http::Request::get(&api_url("/api/v1/buildings")).send().await {
                    if resp.ok() {
                        if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                            let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("address")?.as_str()?.to_string()))).collect();
                            buildings_state.set(mapped);
                        }
                    }
                }
            });
            || ()
        });
    }
    {
        let apartments_state = apartments.clone();
        let selected_building_state = selected_building.clone();
        use_effect_with(selected_building.clone(), move |_| {
            apartments_state.set(Vec::new());
            if let Some(bid) = *selected_building_state {
                let apartments_state2 = apartments_state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/v1/buildings/{}/apartments", bid);
                    if let Ok(resp) = reqwasm::http::Request::get(&api_url(&url)).send().await {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                                let mapped = list.into_iter().filter_map(|v| Some((v.get("id")?.as_u64()?, v.get("building_id")?.as_u64()?, v.get("number")?.as_str()?.to_string()))).collect();
                                apartments_state2.set(mapped);
                            }
                        }
                    }
                });
            }
            || ()
        });
    }

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
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if (*title).trim().is_empty() { error.set(Some(t("announcement-title-label"))); return; }
            if (*body_md).trim().is_empty() { error.set(Some(t("announcement-body-label"))); return; }
            error.set(None);
            saving.set(true);
            if let Some(ex) = &existing {
                // Update existing announcement
                let id = ex.id;
                let roles_string = if selected_roles.is_empty() { None } else { Some(selected_roles.join(",")) };
                let payload = serde_json::json!({
                    "title": (*title).clone(),
                    "body_md": (*body_md).clone(),
                    "public": *public_flag,
                    "pinned": *pinned_flag,
                    "roles_csv": roles_string.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
                    "comments_enabled": *comments_enabled,
                    "publish_at": if publish_at.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(datetime_local_to_naive(&publish_at)) },
                    "expire_at": if expire_at.trim().is_empty() { serde_json::Value::Null } else { serde_json::Value::String(datetime_local_to_naive(&expire_at)) },
                    "building_id": selected_building.as_ref().map(|v| serde_json::Value::from(*v)).unwrap_or(serde_json::Value::Null),
                    "apartment_id": selected_apartment.as_ref().map(|v| serde_json::Value::from(*v)).unwrap_or(serde_json::Value::Null),
                });
                let saving2 = saving.clone();
                let error2 = error.clone();
                let on_updated_cb = on_updated.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::put(&api_url(&format!("/api/v1/announcements/{}", id)))
                        .header("Content-Type", "application/json");
                    if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.body(payload.to_string()).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<AnnouncementFull>().await {
                                    Ok(updated) => { on_updated_cb.emit(updated); }
                                    Err(_) => error2.set(Some(t("error-load-failed")))
                                }
                            } else { error2.set(Some(format!("{} {}", t("error-load-failed"), resp.status()))); }
                            saving2.set(false);
                        }
                        Err(_) => { error2.set(Some(t("error-network"))); saving2.set(false); }
                    }
                });
            } else {
                // Create new
                let roles_csv_opt = if selected_roles.is_empty() { None } else { Some(selected_roles.join(",")) };
                let payload = CreatePayload {
                    title: (*title).clone(),
                    body_md: (*body_md).clone(),
                    public: *public_flag,
                    pinned: *pinned_flag,
                    roles_csv: roles_csv_opt,
                    building_id: *selected_building,
                    apartment_id: *selected_apartment,
                    comments_enabled: *comments_enabled,
                    publish_at: if publish_at.trim().is_empty() { None } else { Some(datetime_local_to_naive(&publish_at)) },
                    expire_at: if expire_at.trim().is_empty() { None } else { Some(datetime_local_to_naive(&expire_at)) },
                };
                let saving2 = saving.clone();
                let error2 = error.clone();
                let on_created_outer = on_created.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut req = reqwasm::http::Request::post(&api_url("/api/v1/announcements"))
                        .header("Content-Type", "application/json");
                    if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                    match req.body(serde_json::to_string(&payload).unwrap()).send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<AnnouncementFull>().await {
                                    Ok(created) => { on_created_outer.emit(created); }
                                    Err(_) => { error2.set(Some(t("error-load-failed"))); }
                                }
                            } else {
                                error2.set(Some(format!("{} ({}): {}", t("error-post-failed"), resp.status(), resp.text().await.unwrap_or_default())));
                            }
                            saving2.set(false);
                        }
                        Err(_) => { error2.set(Some(t("error-network"))); saving2.set(false); }
                    }
                });
            }
        })
    };

    let publish_now_btn = if let Some(ex) = &props.existing { if ex.publish_at.is_none() { Some(ex.id) } else { None } } else { None };
    let on_publish_now = {
        let on_published = props.on_published.clone();
        let saving = saving.clone();
        let error = error.clone();
        Callback::from(move |id: u64| {
            saving.set(true);
            let saving2 = saving.clone();
            let error2 = error.clone();
            let on_published_cb = on_published.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url(&format!("/api/v1/announcements/{}/publish", id)))
                    .header("Content-Type", "application/json");
                if let Some(tok) = get_token() { req = req.header("Authorization", &format!("Bearer {}", tok)); }
                match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(updated) = resp.json::<AnnouncementFull>().await { on_published_cb.emit(updated); }
                        } else { error2.set(Some(format!("{} {}", t("error-post-failed"), resp.status()))); }
                    }
                    Err(_) => { error2.set(Some(t("error-network"))); saving2.set(false); }
                }
            });
        })
    };

    html! {
        <div class="card mb-4">
            <div class="card-header"><strong>{ if props.existing.is_some() { t("button-update") } else { t("button-publish") } }</strong></div>
            <div class="card-body">
                { if let Some(err) = &*error { html!{<div class="alert alert-danger py-1">{err}</div>} } else { html!{<></>} } }
                <form onsubmit={on_submit}>
                    <div class="mb-2">
                        <label class="form-label small">{ t("announcement-title-label") }</label>
                        <input class="form-control form-control-sm" value={(*title).clone()} oninput={{ let s=title.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.value()); }) }} placeholder={t("announcement-title-label")} />
                    </div>
                    <div class="mb-3">
                        <ul class="nav nav-tabs nav-sm">
                            <li class="nav-item">
                                <button type="button" class={classes!("nav-link", if *active_tab=="edit" {"active"} else {""})} onclick={{ let ttab=active_tab.clone(); Callback::from(move |_| ttab.set("edit".into())) }}>{ t("announcement-edit-tab") }</button>
                            </li>
                            <li class="nav-item">
                                <button type="button" class={classes!("nav-link", if *active_tab=="preview" {"active"} else {""})} onclick={{ let ttab=active_tab.clone(); Callback::from(move |_| ttab.set("preview".into())) }}>{ t("announcement-preview-tab") }</button>
                            </li>
                        </ul>
                    </div>
                    <div class="row">
                        { if *active_tab == "edit" { html!{
                            <div class="col-12">
                                <label class="form-label small">{ t("announcement-body-label") }</label>
                                <textarea class="form-control" rows=12 value={(*body_md).clone()} oninput={{ let s=body_md.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlTextAreaElement = e.target_unchecked_into(); s.set(i.value()); }) }}></textarea>
                                <div class="form-text small text-muted">{ t("announcement-edit-tab") }{" -> "}{ t("announcement-preview-tab") }</div>
                            </div>
                        }} else { html!{
                            <div class="col-12">
                                <label class="form-label small d-flex justify-content-between align-items-center">{ t("announcement-preview-tab") }<span class="badge bg-secondary">{ t("preview-rendered") }</span></label>
                                <div class="alert alert-secondary mb-0 p-0 border">
                                    <div class="preview-body p-3" style="min-height:240px">
                                        { Html::from_html_unchecked((*preview_html_computed).clone().into()) }
                                    </div>
                                </div>
                            </div>
                        }} }
                    </div>
                    <div class="row mt-2 g-2">
                        <div class="col-md-4">
                            <label class="form-label small">{ t("announcement-roles-label") }{" (visibility)"}</label>
                            { for ["Admin", "Manager", "Homeowner", "Renter", "HOA Member"].into_iter().map(|role| {
                                let sel = selected_roles.clone();
                                let r = role.to_string();
                                let checked = selected_roles.iter().any(|x| x == role);
                                let label_key = match role {
                                    "Admin" => "role-admin",
                                    "Manager" => "role-manager",
                                    "Homeowner" => "role-homeowner",
                                    "Renter" => "role-renter",
                                    "HOA Member" => "role-hoa-member",
                                    _ => "role-unknown",
                                };
                                html!{<div class="form-check form-check-sm">
                                    <input class="form-check-input" type="checkbox" id={format!("role_{}", role.replace(' ', "_"))} checked={checked}
                                        onchange={Callback::from(move |e: Event| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); let mut list = (*sel).clone(); if input.checked() { if !list.iter().any(|v| v==&r) { list.push(r.clone()); } } else { list.retain(|v| v!=&r); } sel.set(list); })} />
                                    <label class="form-check-label small" for={format!("role_{}", role.replace(' ', "_"))}>{ t(label_key) }</label>
                                </div>}
                            }) }
                            <div class="form-text small text-muted">{ t("announcement-roles-help") }</div>
                        </div>
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-publish-at-label") }</label>
                            <input type="datetime-local" class="form-control form-control-sm" value={(*publish_at).clone()} oninput={{ let s=publish_at.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.value()); }) }} />
                        </div>
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-expire-at-label") }</label>
                            <input type="datetime-local" class="form-control form-control-sm" value={(*expire_at).clone()} oninput={{ let s=expire_at.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.value()); }) }} />
                        </div>
                        <div class="col-md-3">
                            <label class="form-label small">{ t("announcement-building-label") }{" (optional)"}</label>
                            <select class="form-select form-select-sm" onchange={{ let sb=selected_building.clone(); let sa=selected_apartment.clone(); Callback::from(move |e: Event| { let sel: web_sys::HtmlSelectElement = e.target_unchecked_into(); let val = sel.value(); if val.is_empty() { sb.set(None); sa.set(None); } else { if let Ok(parsed)=val.parse::<u64>() { sb.set(Some(parsed)); sa.set(None); } } }) }}>
                                <option value="" selected={selected_building.is_none()}>{ t("none-option") }</option>
                                { for buildings.iter().map(|(id,addr)| html!{<option value={id.to_string()} selected={selected_building.map(|v| v==*id).unwrap_or(false)}>{addr}</option>}) }
                            </select>
                            <label class="form-label small mt-2">{ t("announcement-apartment-label") }</label>
                            <select class="form-select form-select-sm" disabled={selected_building.is_none() || apartments.is_empty()} onchange={{ let sa=selected_apartment.clone(); Callback::from(move |e: Event| { let sel: web_sys::HtmlSelectElement = e.target_unchecked_into(); let val = sel.value(); if val.is_empty() { sa.set(None); } else { if let Ok(parsed)=val.parse::<u64>() { sa.set(Some(parsed)); } } }) }}>
                                <option value="" selected={selected_apartment.is_none()}>{ t("none-option") }</option>
                                { for apartments.iter().filter(|(_,bid,_)| Some(*bid)==*selected_building).map(|(id,_bid,num)| html!{<option value={id.to_string()} selected={selected_apartment.map(|v| v==*id).unwrap_or(false)}>{num}</option>}) }
                            </select>
                        </div>
                        <div class="col-md-2">
                            <label class="form-label small">{ t("announcement-options-heading") }</label>
                            <div class="form-check">
                                <input class="form-check-input" type="checkbox" checked={*public_flag} onchange={{ let s=public_flag.clone(); Callback::from(move |e: Event| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.checked()); }) }} id="annPublic" />
                                <label class="form-check-label small" for="annPublic">{ t("announcement-public-label") }</label>
                            </div>
                            <div class="form-check">
                                <input class="form-check-input" type="checkbox" checked={*pinned_flag} onchange={{ let s=pinned_flag.clone(); Callback::from(move |e: Event| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.checked()); }) }} id="annPinned" />
                                <label class="form-check-label small" for="annPinned">{ t("announcement-pin-label") }</label>
                            </div>
                            <div class="form-check">
                                <input class="form-check-input" type="checkbox" checked={*comments_enabled} onchange={{ let s=comments_enabled.clone(); Callback::from(move |e: Event| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); s.set(i.checked()); }) }} id="annComments" />
                                <label class="form-check-label small" for="annComments">{ t("announcement-comments-enabled-label") }</label>
                            </div>
                        </div>
                    </div>
                    <div class="mt-3 d-flex gap-2">
                        <button class="btn btn-sm btn-primary" type="submit" disabled={*saving}>
                            { if *saving { html!{<Spinner small={true} color={"light"} /> } } else { if props.existing.is_some() { html!{ t("button-save") } } else { html!{ t("button-publish") } } } }
                        </button>
                        { if let Some(id) = publish_now_btn { html!{ <button type="button" class="btn btn-sm btn-success" disabled={*saving} onclick={Callback::from(move |_| on_publish_now.emit(id))}>{ t("announcement-publish-now") }</button> } } else { html!{} } }
                        { if props.existing.is_some() { html!{ <button type="button" class="btn btn-sm btn-outline-secondary" onclick={props.on_cancel.reform(|_|())}>{ t("button-cancel") }</button> } } else { html!{} } }
                    </div>
                </form>
            </div>
        </div>
    }
}

fn datetime_local_to_naive(val: &str) -> String {
    // HTML datetime-local gives YYYY-MM-DDTHH:MM; append :00 seconds if missing
    if val.len() == 16 { format!("{}:00", val) } else { val.to_string() }
}
