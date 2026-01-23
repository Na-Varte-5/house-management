use crate::components::spinner::Spinner;
use crate::i18n::t;
use crate::utils::{
    api::api_url,
    auth::{current_user, get_token},
    datetime::format_dt_local,
};
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct CommentDto {
    pub id: u64,
    pub announcement_id: u64,
    pub user_id: u64,
    pub user_name: String,
    pub body_html: String,
    pub body_md: String,
    pub created_at: Option<String>,
    pub is_deleted: bool,
}

#[derive(Properties, PartialEq)]
pub struct CommentListProps {
    pub announcement_id: u64,
    pub comments_enabled: bool,
}

#[function_component(CommentList)]
pub fn comment_list(props: &CommentListProps) -> Html {
    if !props.comments_enabled {
        return html! {<div class="text-muted small">{ t("comments-disabled") }</div>};
    }

    let loading = use_state(|| false);
    let comments = use_state(|| Vec::<CommentDto>::new());
    let error = use_state(|| None::<String>);
    let posting = use_state(|| false);
    let new_md = use_state(String::default);
    let show_deleted = use_state(|| false);
    let current = current_user();
    let can_post = current.is_some();
    let is_manager = current
        .as_ref()
        .map(|c| c.roles.iter().any(|r| r == "Admin" || r == "Manager"))
        .unwrap_or(false);

    // Load comments when announcement_id changes
    {
        let loading = loading.clone();
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_state = props.announcement_id;
        let show_deleted_val = *show_deleted; // capture bool for effect
        use_effect_with((props.announcement_id, *show_deleted), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let mut url = api_url(&format!("/api/v1/announcements/{}/comments", ann_id_state));
                if show_deleted_val {
                    url.push_str("?include_deleted=true");
                }
                let mut req = reqwasm::http::Request::get(&url);
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<serde_json::Value>>().await {
                                let mapped = list
                                    .into_iter()
                                    .filter_map(|v| {
                                        Some(CommentDto {
                                            id: v.get("id")?.as_u64()?,
                                            announcement_id: v.get("announcement_id")?.as_u64()?,
                                            user_id: v.get("user_id")?.as_u64()?,
                                            user_name: v.get("user_name")?.as_str()?.to_string(),
                                            body_html: v.get("body_html")?.as_str()?.to_string(),
                                            body_md: v.get("body_md")?.as_str()?.to_string(),
                                            created_at: v
                                                .get("created_at")
                                                .and_then(|x| x.as_str())
                                                .map(|s| s.to_string()),
                                            is_deleted: v.get("is_deleted")?.as_bool()?,
                                        })
                                    })
                                    .collect();
                                comments_state.set(mapped);
                            }
                        } else {
                            error_state.set(Some(format!(
                                "{}: {}",
                                t("error-load-failed"),
                                resp.status()
                            )));
                        }
                        loading.set(false);
                    }
                    Err(_) => {
                        error_state.set(Some(t("error-network")));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    // Post comment
    let on_post = {
        let new_md = new_md.clone();
        let comments_state = comments.clone();
        let posting_state = posting.clone();
        let error_state = error.clone();
        let ann_id = props.announcement_id;
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if new_md.trim().is_empty() {
                return;
            }
            posting_state.set(true);
            let body_val = (*new_md).clone();
            let new_md2 = new_md.clone();
            let comments_state2 = comments_state.clone();
            let posting_state2 = posting_state.clone();
            let error_state2 = error_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let payload = serde_json::json!({"body_md": body_val});
                let mut req = reqwasm::http::Request::post(&api_url(&format!(
                    "/api/v1/announcements/{}/comments",
                    ann_id
                )))
                .header("Content-Type", "application/json");
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                let send_res = req.body(payload.to_string()).send().await;
                if let Ok(resp) = send_res {
                    if resp.ok() {
                        let mut req2 = reqwasm::http::Request::get(&api_url(&format!(
                            "/api/v1/announcements/{}/comments",
                            ann_id
                        )));
                        if let Some(tok) = get_token() {
                            req2 = req2.header("Authorization", &format!("Bearer {}", tok));
                        }
                        if let Ok(r2) = req2.send().await {
                            if r2.ok() {
                                if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                                    let mapped = list
                                        .into_iter()
                                        .filter_map(|v| {
                                            Some(CommentDto {
                                                id: v.get("id")?.as_u64()?,
                                                announcement_id: v
                                                    .get("announcement_id")?
                                                    .as_u64()?,
                                                user_id: v.get("user_id")?.as_u64()?,
                                                user_name: v
                                                    .get("user_name")?
                                                    .as_str()?
                                                    .to_string(),
                                                body_html: v
                                                    .get("body_html")?
                                                    .as_str()?
                                                    .to_string(),
                                                body_md: v.get("body_md")?.as_str()?.to_string(),
                                                created_at: v
                                                    .get("created_at")
                                                    .and_then(|x| x.as_str())
                                                    .map(|s| s.to_string()),
                                                is_deleted: v.get("is_deleted")?.as_bool()?,
                                            })
                                        })
                                        .collect();
                                    comments_state2.set(mapped);
                                }
                            }
                        }
                        new_md2.set(String::new());
                    } else {
                        error_state2.set(Some(t("comment-post-failed")));
                    }
                } else {
                    error_state2.set(Some(t("error-network")));
                }
                posting_state2.set(false);
            });
        })
    };

    let delete_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_delete = props.announcement_id;
        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::delete(&api_url(&format!(
                    "/api/v1/announcements/comments/{}",
                    comment_id
                )));
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(resp) = req.send().await {
                    if !resp.ok() {
                        error_state2.set(Some(t("comment-delete-failed")));
                    }
                }
                // refresh
                let mut req2 = reqwasm::http::Request::get(&api_url(&format!(
                    "/api/v1/announcements/{}/comments",
                    ann_id_for_delete
                )));
                if let Some(tok) = get_token() {
                    req2 = req2.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(r2) = req2.send().await {
                    if r2.ok() {
                        if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                            let mapped = list
                                .into_iter()
                                .filter_map(|v| {
                                    Some(CommentDto {
                                        id: v.get("id")?.as_u64()?,
                                        announcement_id: v.get("announcement_id")?.as_u64()?,
                                        user_id: v.get("user_id")?.as_u64()?,
                                        user_name: v.get("user_name")?.as_str()?.to_string(),
                                        body_html: v.get("body_html")?.as_str()?.to_string(),
                                        body_md: v.get("body_md")?.as_str()?.to_string(),
                                        created_at: v
                                            .get("created_at")
                                            .and_then(|x| x.as_str())
                                            .map(|s| s.to_string()),
                                        is_deleted: v.get("is_deleted")?.as_bool()?,
                                    })
                                })
                                .collect();
                            comments_state2.set(mapped);
                        }
                    }
                }
            });
        })
    };
    let restore_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_restore = props.announcement_id;
        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::post(&api_url(&format!(
                    "/api/v1/announcements/comments/{}/restore",
                    comment_id
                )));
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(resp) = req.send().await {
                    if !resp.ok() {
                        error_state2.set(Some(t("comment-restore-failed")));
                    }
                }
                let mut req2 = reqwasm::http::Request::get(&api_url(&format!(
                    "/api/v1/announcements/{}/comments?include_deleted=true",
                    ann_id_for_restore
                )));
                if let Some(tok) = get_token() {
                    req2 = req2.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(r2) = req2.send().await {
                    if r2.ok() {
                        if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                            let mapped = list
                                .into_iter()
                                .filter_map(|v| {
                                    Some(CommentDto {
                                        id: v.get("id")?.as_u64()?,
                                        announcement_id: v.get("announcement_id")?.as_u64()?,
                                        user_id: v.get("user_id")?.as_u64()?,
                                        user_name: v.get("user_name")?.as_str()?.to_string(),
                                        body_html: v.get("body_html")?.as_str()?.to_string(),
                                        body_md: v.get("body_md")?.as_str()?.to_string(),
                                        created_at: v
                                            .get("created_at")
                                            .and_then(|x| x.as_str())
                                            .map(|s| s.to_string()),
                                        is_deleted: v.get("is_deleted")?.as_bool()?,
                                    })
                                })
                                .collect();
                            comments_state2.set(mapped);
                        }
                    }
                }
            });
        })
    };
    let purge_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_restore = props.announcement_id;
        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut req = reqwasm::http::Request::delete(&api_url(&format!(
                    "/api/v1/announcements/comments/{}/purge",
                    comment_id
                )));
                if let Some(tok) = get_token() {
                    req = req.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(resp) = req.send().await {
                    if !resp.ok() {
                        error_state2.set(Some(t("comment-purge-failed")));
                    }
                }
                let mut req2 = reqwasm::http::Request::get(&api_url(&format!(
                    "/api/v1/announcements/{}/comments?include_deleted=true",
                    ann_id_for_restore
                )));
                if let Some(tok) = get_token() {
                    req2 = req2.header("Authorization", &format!("Bearer {}", tok));
                }
                if let Ok(r2) = req2.send().await {
                    if r2.ok() {
                        if let Ok(list) = r2.json::<Vec<serde_json::Value>>().await {
                            let mapped = list
                                .into_iter()
                                .filter_map(|v| {
                                    Some(CommentDto {
                                        id: v.get("id")?.as_u64()?,
                                        announcement_id: v.get("announcement_id")?.as_u64()?,
                                        user_id: v.get("user_id")?.as_u64()?,
                                        user_name: v.get("user_name")?.as_str()?.to_string(),
                                        body_html: v.get("body_html")?.as_str()?.to_string(),
                                        body_md: v.get("body_md")?.as_str()?.to_string(),
                                        created_at: v
                                            .get("created_at")
                                            .and_then(|x| x.as_str())
                                            .map(|s| s.to_string()),
                                        is_deleted: v.get("is_deleted")?.as_bool()?,
                                    })
                                })
                                .collect();
                            comments_state2.set(mapped);
                        }
                    }
                }
            });
        })
    };

    let comments_view: Html = if *loading {
        html! {<Spinner center={true} />}
    } else if comments.is_empty() {
        html! {<div class="text-muted small">{ t("comments-empty") }</div>}
    } else {
        let rendered: Vec<Html> = comments.iter().cloned().map(|c| {
            let del_cb = delete_comment.clone();
            let res_cb = restore_comment.clone();
            let pur_cb = purge_comment.clone();
            html! {<div class="mb-2">
                <div class={classes!("p-2","border","rounded", if c.is_deleted {"bg-light"} else {"bg-white"})}>{ Html::from_html_unchecked(c.body_html.clone().into()) }
                    { if c.is_deleted { html!{<span class="badge bg-danger ms-2">{ t("comment-deleted-badge") }</span>} } else { html!{} } }
                </div>
                <div class="d-flex justify-content-between small text-muted">
                    <span>{c.created_at.clone().map(|s| format_dt_local(&s)).unwrap_or_default()} {" • "}{c.user_name.clone()}</span>
                    { if is_manager { html!{<div class="btn-group btn-group-sm">
                        { if !c.is_deleted { html!{<button class="btn btn-outline-danger" onclick={Callback::from(move |_| del_cb.emit(c.id))}>{ t("comment-delete-button") }</button>} } else { html!{<><button class="btn btn-outline-success" onclick={Callback::from(move |_| res_cb.emit(c.id))}>{ t("comment-restore-button") }</button><button class="btn btn-outline-danger" onclick={Callback::from(move |_| pur_cb.emit(c.id))}>{ t("comment-purge-button") }</button></>} }
                    } </div>} } else { html!{<></>} } }
                </div>
            </div>}
        }).collect();
        html! { { for rendered } }
    };

    html! {
    <div class="comment-section mt-3">
        <h6 class="border-bottom pb-1">{ t("comments-heading") }</h6>
        { if let Some(err) = &*error { html!{<div class="alert alert-danger py-1">{err}</div>} } else { html!{<></>} } }
        { if is_manager { html!{<div class="form-check form-switch mb-2"><input class="form-check-input" type="checkbox" id="showDeletedComments" checked={*show_deleted} onchange={{ let sd=show_deleted.clone(); Callback::from(move |e: Event| { let i: web_sys::HtmlInputElement = e.target_unchecked_into(); sd.set(i.checked()); }) }} /><label class="form-check-label small" for="showDeletedComments">{ t("comment-show-deleted-toggle") }</label></div>} } else { html!{} } }
        { comments_view }
        { if can_post { html!{
            <form class="mt-2" onsubmit={on_post}>
                <textarea class="form-control form-control-sm" rows=3 placeholder={ t("comment-add-placeholder") } value={(*new_md).clone()} oninput={{ let s=new_md.clone(); Callback::from(move |e: InputEvent| { let i: web_sys::HtmlTextAreaElement = e.target_unchecked_into(); s.set(i.value()); }) }}></textarea>
                <div class="mt-1"><button class="btn btn-sm btn-secondary" disabled={*posting} type="submit">{ if *posting { html!{<Spinner small={true} color={"light"} /> } } else { html!{ t("comment-post-button") } } }</button></div>
            </form>
        } } else { html!{<div class="text-muted small mt-2">{ t("comments-login-to-comment") }</div>} }  } </div> }
}
