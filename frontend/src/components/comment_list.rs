use crate::components::spinner::Spinner;
use crate::contexts::AuthContext;
use crate::i18n::t;
use crate::services::api_client;
use crate::utils::datetime::format_dt_local;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
struct PostCommentRequest {
    body_md: String,
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

    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let loading = use_state(|| false);
    let comments = use_state(|| Vec::<CommentDto>::new());
    let error = use_state(|| None::<String>);
    let posting = use_state(|| false);
    let new_md = use_state(String::default);
    let show_deleted = use_state(|| false);
    let can_post = auth.is_authenticated();
    let is_manager = auth.is_admin_or_manager();
    let token = auth.token().map(|t| t.to_string());

    // Load comments when announcement_id or show_deleted changes
    {
        let loading = loading.clone();
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_state = props.announcement_id;
        let show_deleted_val = *show_deleted;
        let token = token.clone();

        use_effect_with((props.announcement_id, *show_deleted), move |_| {
            loading.set(true);
            error_state.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                let endpoint = if show_deleted_val {
                    format!(
                        "/announcements/{}/comments?include_deleted=true",
                        ann_id_state
                    )
                } else {
                    format!("/announcements/{}/comments", ann_id_state)
                };

                match client.get::<Vec<CommentDto>>(&endpoint).await {
                    Ok(list) => comments_state.set(list),
                    Err(e) => error_state.set(Some(format!("{}: {}", t("error-load-failed"), e))),
                }
                loading.set(false);
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
        let token = token.clone();
        let show_deleted_val = *show_deleted;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if new_md.trim().is_empty() {
                return;
            }

            posting_state.set(true);
            error_state.set(None);

            let body_val = (*new_md).clone();
            let new_md2 = new_md.clone();
            let comments_state2 = comments_state.clone();
            let posting_state2 = posting_state.clone();
            let error_state2 = error_state.clone();
            let token2 = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token2.as_deref());
                let payload = PostCommentRequest { body_md: body_val };
                let endpoint = format!("/announcements/{}/comments", ann_id);

                match client
                    .post::<_, serde_json::Value>(&endpoint, &payload)
                    .await
                {
                    Ok(_) => {
                        // Reload comments after successful post
                        let reload_endpoint = if show_deleted_val {
                            format!("/announcements/{}/comments?include_deleted=true", ann_id)
                        } else {
                            format!("/announcements/{}/comments", ann_id)
                        };

                        if let Ok(list) = client.get::<Vec<CommentDto>>(&reload_endpoint).await {
                            comments_state2.set(list);
                        }
                        new_md2.set(String::new());
                    }
                    Err(_) => error_state2.set(Some(t("comment-post-failed"))),
                }
                posting_state2.set(false);
            });
        })
    };

    // Delete comment
    let delete_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_delete = props.announcement_id;
        let token = token.clone();
        let show_deleted_val = *show_deleted;

        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            let token2 = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token2.as_deref());
                let delete_endpoint = format!("/announcements/comments/{}", comment_id);

                match client.delete_no_response(&delete_endpoint).await {
                    Ok(_) => {}
                    Err(_) => {
                        error_state2.set(Some(t("comment-delete-failed")));
                        return;
                    }
                }

                // Reload comments
                let reload_endpoint = if show_deleted_val {
                    format!(
                        "/announcements/{}/comments?include_deleted=true",
                        ann_id_for_delete
                    )
                } else {
                    format!("/announcements/{}/comments", ann_id_for_delete)
                };

                if let Ok(list) = client.get::<Vec<CommentDto>>(&reload_endpoint).await {
                    comments_state2.set(list);
                }
            });
        })
    };

    // Restore comment
    let restore_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_restore = props.announcement_id;
        let token = token.clone();

        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            let token2 = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token2.as_deref());
                let restore_endpoint = format!("/announcements/comments/{}/restore", comment_id);

                match client
                    .post_empty::<serde_json::Value>(&restore_endpoint)
                    .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        error_state2.set(Some(t("comment-restore-failed")));
                        return;
                    }
                }

                // Reload comments with deleted items visible
                let reload_endpoint = format!(
                    "/announcements/{}/comments?include_deleted=true",
                    ann_id_for_restore
                );

                if let Ok(list) = client.get::<Vec<CommentDto>>(&reload_endpoint).await {
                    comments_state2.set(list);
                }
            });
        })
    };

    // Purge comment
    let purge_comment = {
        let comments_state = comments.clone();
        let error_state = error.clone();
        let ann_id_for_purge = props.announcement_id;
        let token = token.clone();

        Callback::from(move |comment_id: u64| {
            let comments_state2 = comments_state.clone();
            let error_state2 = error_state.clone();
            let token2 = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token2.as_deref());
                let purge_endpoint = format!("/announcements/comments/{}/purge", comment_id);

                match client.delete_no_response(&purge_endpoint).await {
                    Ok(_) => {}
                    Err(_) => {
                        error_state2.set(Some(t("comment-purge-failed")));
                        return;
                    }
                }

                // Reload comments with deleted items visible
                let reload_endpoint = format!(
                    "/announcements/{}/comments?include_deleted=true",
                    ann_id_for_purge
                );

                if let Ok(list) = client.get::<Vec<CommentDto>>(&reload_endpoint).await {
                    comments_state2.set(list);
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
                <div class={classes!("p-2","border","rounded", if c.is_deleted {"bg-light"} else {"bg-white"})}>
                    { Html::from_html_unchecked(c.body_html.clone().into()) }
                    { if c.is_deleted { html!{<span class="badge bg-danger ms-2">{ t("comment-deleted-badge") }</span>} } else { html!{} } }
                </div>
                <div class="d-flex justify-content-between small text-muted">
                    <span>
                        {c.created_at.clone().map(|s| format_dt_local(&s)).unwrap_or_default()}
                        {" • "}
                        {c.user_name.clone()}
                    </span>
                    { if is_manager {
                        html!{
                            <div class="btn-group btn-group-sm">
                                { if !c.is_deleted {
                                    html!{
                                        <button
                                            class="btn btn-outline-danger"
                                            onclick={Callback::from(move |_| del_cb.emit(c.id))}
                                        >
                                            { t("comment-delete-button") }
                                        </button>
                                    }
                                } else {
                                    html!{
                                        <>
                                            <button
                                                class="btn btn-outline-success"
                                                onclick={Callback::from(move |_| res_cb.emit(c.id))}
                                            >
                                                { t("comment-restore-button") }
                                            </button>
                                            <button
                                                class="btn btn-outline-danger"
                                                onclick={Callback::from(move |_| pur_cb.emit(c.id))}
                                            >
                                                { t("comment-purge-button") }
                                            </button>
                                        </>
                                    }
                                } }
                            </div>
                        }
                    } else {
                        html!{<></>}
                    } }
                </div>
            </div>}
        }).collect();
        html! { { for rendered } }
    };

    html! {
        <div class="comment-section mt-3">
            <h6 class="border-bottom pb-1">{ t("comments-heading") }</h6>

            { if let Some(err) = &*error {
                html!{<div class="alert alert-danger py-1">{err}</div>}
            } else {
                html!{<></>}
            } }

            { if is_manager {
                let sd=show_deleted.clone();
                html!{
                    <div class="form-check form-switch mb-2">
                        <input
                            class="form-check-input"
                            type="checkbox"
                            id="showDeletedComments"
                            checked={*show_deleted}
                            onchange={Callback::from(move |e: Event| {
                                let i: web_sys::HtmlInputElement = e.target_unchecked_into();
                                sd.set(i.checked());
                            })}
                        />
                        <label class="form-check-label small" for="showDeletedComments">
                            { t("comment-show-deleted-toggle") }
                        </label>
                    </div>
                }
            } else {
                html!{}
            } }

            { comments_view }

            { if can_post {
                let s=new_md.clone();
                html!{
                    <form class="mt-2" onsubmit={on_post}>
                        <textarea
                            class="form-control form-control-sm"
                            rows=3
                            placeholder={ t("comment-add-placeholder") }
                            value={(*new_md).clone()}
                            oninput={Callback::from(move |e: InputEvent| {
                                let i: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                s.set(i.value());
                            })}
                        />
                        <div class="mt-1">
                            <button class="btn btn-sm btn-secondary" disabled={*posting} type="submit">
                                { if *posting {
                                    html!{<Spinner small={true} color={"light"} />}
                                } else {
                                    html!{ t("comment-post-button") }
                                } }
                            </button>
                        </div>
                    </form>
                }
            } else {
                html!{<div class="text-muted small mt-2">{ t("comments-login-to-comment") }</div>}
            } }
        </div>
    }
}
