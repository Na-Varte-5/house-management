use crate::components::forms::Textarea;
use crate::i18n::t;
use crate::utils::datetime::format_dt_option;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Comment {
    pub id: u64,
    pub request_id: u64,
    pub user_id: u64,
    pub user_name: String,
    pub comment_text: String,
    pub is_deleted: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct CommentSectionProps {
    pub comments: Vec<Comment>,
    pub loading: bool,
    pub request_id: u64,
    pub current_user_id: u64,
    pub is_admin_or_manager: bool,
    pub on_add_comment: Callback<String>,
    pub on_delete_comment: Callback<u64>,
}

/// CommentSection component for displaying and adding comments
#[function_component(CommentSection)]
pub fn comment_section(props: &CommentSectionProps) -> Html {
    let comment_text = use_state(|| String::new());
    let submitting = use_state(|| false);

    let on_text_change = {
        let comment_text = comment_text.clone();
        Callback::from(move |value: String| {
            comment_text.set(value);
        })
    };

    let on_submit = {
        let comment_text = comment_text.clone();
        let submitting = submitting.clone();
        let on_add_comment = props.on_add_comment.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if !comment_text.is_empty() && !*submitting {
                submitting.set(true);
                on_add_comment.emit((*comment_text).clone());
                comment_text.set(String::new());
                // Note: submitting will be reset by parent when comments reload
            }
        })
    };

    // Reset submitting when loading changes to false (comment added successfully)
    {
        let submitting = submitting.clone();
        let loading = props.loading;
        use_effect_with(loading, move |loading| {
            if !loading {
                submitting.set(false);
            }
        });
    }

    html! {
        <div class="card mt-3">
            <div class="card-header">
                <h5 class="mb-0">
                    <i class="bi bi-chat-dots me-2"></i>
                    {t("maintenance-comments")}
                </h5>
            </div>
            <div class="card-body">
                if props.loading && props.comments.is_empty() {
                    <div class="text-center py-3">
                        <div class="spinner-border spinner-border-sm" role="status"></div>
                        <span class="ms-2">{t("maintenance-loading-comments")}</span>
                    </div>
                } else if props.comments.is_empty() {
                    <p class="text-muted mb-0">{t("maintenance-no-comments")}</p>
                } else {
                    <div class="comments-list">
                        { for props.comments.iter().map(|comment| render_comment(comment, props.current_user_id, props.is_admin_or_manager, props.on_delete_comment.clone())) }
                    </div>
                }

                // Add comment form
                <div class="mt-4">
                    <h6>{t("maintenance-add-comment")}</h6>
                    <form onsubmit={on_submit}>
                        <Textarea
                            label=""
                            value={(*comment_text).clone()}
                            on_change={on_text_change}
                            placeholder={t("maintenance-comment-placeholder")}
                            rows={3}
                            required={true}
                        />
                        <button
                            type="submit"
                            class="btn btn-primary btn-sm mt-2"
                            disabled={comment_text.is_empty() || *submitting}
                        >
                            if *submitting {
                                <>
                                    <span class="spinner-border spinner-border-sm me-2" role="status"></span>
                                    {t("maintenance-posting")}
                                </>
                            } else {
                                <>
                                    <i class="bi bi-send me-2"></i>
                                    {t("maintenance-post-comment")}
                                </>
                            }
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}

fn render_comment(
    comment: &Comment,
    current_user_id: u64,
    is_admin_or_manager: bool,
    on_delete: Callback<u64>,
) -> Html {
    let can_delete = is_admin_or_manager || comment.user_id == current_user_id;
    let comment_id = comment.id;
    let on_delete_click = {
        let on_delete = on_delete.clone();
        Callback::from(move |_: MouseEvent| {
            if web_sys::window()
                .and_then(|w| {
                    w.confirm_with_message(&t("maintenance-delete-comment-confirm"))
                        .ok()
                })
                .unwrap_or(false)
            {
                on_delete.emit(comment_id);
            }
        })
    };

    html! {
        <div class="comment-item border-bottom pb-3 mb-3">
            <div class="d-flex justify-content-between align-items-start">
                <div class="flex-grow-1">
                    <div class="d-flex align-items-center mb-1">
                        <strong class="me-2">{&comment.user_name}</strong>
                        <small class="text-muted">{format_dt_option(comment.created_at.as_ref())}</small>
                    </div>
                    <p class="mb-0" style="white-space: pre-wrap;">{&comment.comment_text}</p>
                </div>
                if can_delete {
                    <button
                        class="btn btn-sm btn-outline-danger ms-2"
                        onclick={on_delete_click}
                        title="Delete comment"
                    >
                        <i class="bi bi-trash"></i>
                    </button>
                }
            </div>
        </div>
    }
}
