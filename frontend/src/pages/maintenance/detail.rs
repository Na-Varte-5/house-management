use crate::components::maintenance::{
    Attachment, AttachmentsList, Comment, CommentSection, HistoryEntry, HistoryTimeline,
    ManagementPanel, ManagementRequest, UserInfo,
};
use crate::components::{ErrorAlert, SuccessAlert};
use crate::contexts::AuthContext;
use crate::routes::Route;
use crate::services::api_client;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
struct MaintenanceRequest {
    id: u64,
    apartment_id: u64,
    apartment_number: String,
    building_id: u64,
    building_address: String,
    request_type: String,
    priority: String,
    title: String,
    description: String,
    status: String,
    resolution_notes: Option<String>,
    created_by: u64,
    created_by_name: String,
    assigned_to: Option<u64>,
    assigned_to_name: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: u64,
}

// Helper function to format datetime strings to be more user-friendly
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
                _ => month,
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

#[function_component(MaintenanceDetailPage)]
pub fn maintenance_detail_page(props: &Props) -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let navigator = use_navigator().unwrap();

    let request = use_state(|| None::<MaintenanceRequest>);
    let history = use_state(|| Vec::<HistoryEntry>::new());
    let attachments = use_state(|| Vec::<Attachment>::new());
    let comments = use_state(|| Vec::<Comment>::new());
    let users = use_state(|| Vec::<UserInfo>::new());

    let loading = use_state(|| true);
    let loading_history = use_state(|| false);
    let loading_attachments = use_state(|| false);
    let loading_comments = use_state(|| false);

    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let request_id = props.id;
    let token = auth.token().map(|t| t.to_string());

    // Load request details
    {
        let request = request.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .get::<MaintenanceRequest>(&format!("/requests/{}", id))
                    .await
                {
                    Ok(req) => {
                        request.set(Some(req));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load request: {}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    // Load history
    {
        let history = history.clone();
        let loading_history = loading_history.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            loading_history.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client
                    .get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", id))
                    .await
                {
                    history.set(list);
                }
                loading_history.set(false);
            });
            || ()
        });
    }

    // Load attachments
    {
        let attachments = attachments.clone();
        let loading_attachments = loading_attachments.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            loading_attachments.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client
                    .get::<Vec<Attachment>>(&format!("/requests/{}/attachments", id))
                    .await
                {
                    attachments.set(list);
                }
                loading_attachments.set(false);
            });
            || ()
        });
    }

    // Load comments
    {
        let comments = comments.clone();
        let loading_comments = loading_comments.clone();
        let token = token.clone();

        use_effect_with(request_id, move |id| {
            let id = *id;
            loading_comments.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                if let Ok(list) = client
                    .get::<Vec<Comment>>(&format!("/requests/{}/comments", id))
                    .await
                {
                    comments.set(list);
                }
                loading_comments.set(false);
            });
            || ()
        });
    }

    // Load users for assignment (Admin/Manager only)
    {
        let users = users.clone();
        let token = token.clone();
        let is_admin_or_manager = auth.is_admin_or_manager();

        use_effect_with((), move |_| {
            if is_admin_or_manager {
                wasm_bindgen_futures::spawn_local(async move {
                    let client = api_client(token.as_deref());
                    if let Ok(list) = client.get::<Vec<UserInfo>>("/users/public").await {
                        users.set(list);
                    }
                });
            }
            || ()
        });
    }

    let on_update = {
        let request = request.clone();
        let history = history.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let request = request.clone();
            let history = history.clone();
            let token = token.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                // Reload request details
                if let Ok(req) = client
                    .get::<MaintenanceRequest>(&format!("/requests/{}", request_id))
                    .await
                {
                    request.set(Some(req));
                }
                // Reload history
                if let Ok(list) = client
                    .get::<Vec<HistoryEntry>>(&format!("/requests/{}/history", request_id))
                    .await
                {
                    history.set(list);
                }
            });
        })
    };

    let on_error = {
        let error = error.clone();
        Callback::from(move |msg: String| error.set(Some(msg)))
    };

    let on_success = {
        let success = success.clone();
        Callback::from(move |msg: String| success.set(Some(msg)))
    };

    let on_add_comment = {
        let comments = comments.clone();
        let loading_comments = loading_comments.clone();
        let success = success.clone();
        let error = error.clone();
        let token = token.clone();

        Callback::from(move |comment_text: String| {
            let comments = comments.clone();
            let loading_comments = loading_comments.clone();
            let success = success.clone();
            let error = error.clone();
            let token = token.clone();

            loading_comments.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                #[derive(serde::Serialize)]
                struct NewComment {
                    comment_text: String,
                }
                match client
                    .post::<NewComment, Comment>(
                        &format!("/requests/{}/comments", request_id),
                        &NewComment { comment_text },
                    )
                    .await
                {
                    Ok(_) => {
                        success.set(Some("Comment posted successfully".to_string()));
                        // Reload comments
                        if let Ok(list) = client
                            .get::<Vec<Comment>>(&format!("/requests/{}/comments", request_id))
                            .await
                        {
                            comments.set(list);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to post comment: {}", e)));
                    }
                }
                loading_comments.set(false);
            });
        })
    };

    let on_delete_comment = {
        let comments = comments.clone();
        let loading_comments = loading_comments.clone();
        let success = success.clone();
        let error = error.clone();
        let token = token.clone();

        Callback::from(move |comment_id: u64| {
            let comments = comments.clone();
            let loading_comments = loading_comments.clone();
            let success = success.clone();
            let error = error.clone();
            let token = token.clone();

            loading_comments.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client
                    .delete_no_response(&format!(
                        "/requests/{}/comments/{}",
                        request_id, comment_id
                    ))
                    .await
                {
                    Ok(_) => {
                        success.set(Some("Comment deleted successfully".to_string()));
                        // Reload comments
                        if let Ok(list) = client
                            .get::<Vec<Comment>>(&format!("/requests/{}/comments", request_id))
                            .await
                        {
                            comments.set(list);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to delete comment: {}", e)));
                    }
                }
                loading_comments.set(false);
            });
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Maintenance);
        })
    };

    let clear_error = {
        let error = error.clone();
        Callback::from(move |_| error.set(None))
    };

    let clear_success = {
        let success = success.clone();
        Callback::from(move |_| success.set(None))
    };

    // Priority badge color
    let priority_class = |priority: &str| match priority {
        "Urgent" => "bg-danger",
        "High" => "bg-warning text-dark",
        "Medium" => "bg-info",
        "Low" => "bg-secondary",
        _ => "bg-light text-dark",
    };

    // Status badge color
    let status_class = |status: &str| match status {
        "Open" => "bg-primary",
        "InProgress" => "bg-warning text-dark",
        "Resolved" => "bg-success",
        _ => "bg-secondary",
    };

    html! {
        <div class="container mt-4">
            <div class="mb-3">
                <button class="btn btn-outline-secondary btn-sm" onclick={on_back}>
                    {"← Back to List"}
                </button>
            </div>

            if let Some(err) = (*error).clone() {
                <ErrorAlert message={err} on_close={clear_error.clone()} />
            }

            if let Some(msg) = (*success).clone() {
                <SuccessAlert message={msg} on_close={clear_success.clone()} />
            }

            if *loading {
                <div class="text-center py-5">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                </div>
            } else if let Some(req) = (*request).clone() {
                <div class="row">
                    // Main details
                    <div class="col-lg-8 mb-3">
                        <div class="card">
                            <div class="card-header">
                                <h4 class="mb-0">{&req.title}</h4>
                            </div>
                            <div class="card-body">
                                <div class="mb-3">
                                    <div class="d-flex gap-2 mb-2">
                                        <span class={classes!("badge", status_class(&req.status))}>
                                            {&req.status}
                                        </span>
                                        <span class={classes!("badge", priority_class(&req.priority))}>
                                            {&req.priority}
                                        </span>
                                        <span class="badge bg-light text-dark">
                                            {&req.request_type}
                                        </span>
                                    </div>
                                </div>

                                <div class="mb-3">
                                    <h6 class="text-muted small">{"Description"}</h6>
                                    <p class="mb-0">{&req.description}</p>
                                </div>

                                <hr />

                                <div class="row small text-muted">
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Apartment:"}</strong>{" "}{format!("{} ({})", req.apartment_number, req.building_address)}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Created by:"}</strong>{" "}{&req.created_by_name}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Assigned to:"}</strong>{" "}
                                        {req.assigned_to_name.as_ref().map(|name| name.clone()).unwrap_or_else(|| "Unassigned".to_string())}
                                    </div>
                                    <div class="col-md-6 mb-2">
                                        <strong>{"Created:"}</strong>{" "}{format_date(&req.created_at)}
                                    </div>
                                </div>
                            </div>
                        </div>

                        // History component
                        <HistoryTimeline
                            history={(*history).clone()}
                            loading={*loading_history}
                        />

                        // Attachments component
                        <AttachmentsList
                            attachments={(*attachments).clone()}
                            loading={*loading_attachments}
                        />

                        // Comments component
                        <CommentSection
                            comments={(*comments).clone()}
                            loading={*loading_comments}
                            request_id={request_id}
                            current_user_id={auth.user().map(|u| u.id).unwrap_or(0)}
                            is_admin_or_manager={auth.is_admin_or_manager()}
                            on_add_comment={on_add_comment}
                            on_delete_comment={on_delete_comment}
                        />
                    </div>

                    // Management Panel component (Admin/Manager only)
                    if auth.is_admin_or_manager() {
                        <div class="col-lg-4 mb-3">
                            <ManagementPanel
                                request={ManagementRequest {
                                    id: req.id,
                                    status: req.status.clone(),
                                    priority: req.priority.clone(),
                                    assigned_to: req.assigned_to,
                                }}
                                users={(*users).clone()}
                                token={token.clone()}
                                on_update={on_update}
                                on_error={on_error}
                                on_success={on_success}
                            />
                        </div>
                    }
                </div>
            } else {
                <div class="alert alert-warning">
                    {"Request not found"}
                </div>
            }
        </div>
    }
}
