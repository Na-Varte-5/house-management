use super::types::CommentRow;
use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{
    CreateCommentRequest, MaintenanceRequest, MaintenanceRequestComment,
    MaintenanceRequestCommentWithUser, NewMaintenanceRequestComment,
};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

/// List comments for a maintenance request
///
/// Returns all active (non-deleted) comments for a maintenance request with user names.
/// Users can view comments if they created the request, are assigned to it, or are Admin/Manager.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/comments",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "List of comments", body = Vec<MaintenanceRequestCommentWithUser>),
        (status = 403, description = "Forbidden - not authorized to view comments"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn list_comments(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_comments::dsl as mrc;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    let request_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let request: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(request_id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;

    let user_id = auth.user_id()?;

    let can_view = auth.has_any_role(&["Admin", "Manager"])
        || request.created_by == user_id
        || request.assigned_to == Some(user_id);

    if !can_view {
        return Err(AppError::Forbidden);
    }

    let comments: Vec<CommentRow> = mrc::maintenance_request_comments
        .inner_join(u::users)
        .filter(mrc::request_id.eq(request_id))
        .filter(mrc::is_deleted.eq(false))
        .select((
            mrc::id,
            mrc::request_id,
            mrc::user_id,
            mrc::comment_text,
            mrc::is_deleted,
            mrc::created_at,
            mrc::updated_at,
            u::name,
        ))
        .order_by(mrc::created_at.asc())
        .load(&mut conn)?;

    let enriched: Vec<MaintenanceRequestCommentWithUser> = comments
        .into_iter()
        .map(
            |(
                id,
                request_id,
                user_id,
                comment_text,
                is_deleted,
                created_at,
                updated_at,
                user_name,
            )| {
                MaintenanceRequestCommentWithUser {
                    id,
                    request_id,
                    user_id,
                    user_name,
                    comment_text,
                    is_deleted,
                    created_at,
                    updated_at,
                }
            },
        )
        .collect();

    Ok(HttpResponse::Ok().json(enriched))
}

/// Create a comment on a maintenance request
///
/// Creates a new comment on a maintenance request.
/// Users can comment if they created the request, are assigned to it, or are Admin/Manager.
#[utoipa::path(
    post,
    path = "/api/v1/requests/{id}/comments",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created successfully", body = MaintenanceRequestCommentWithUser),
        (status = 403, description = "Forbidden - not authorized to comment"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn create_comment(
    auth: AuthContext,
    path: web::Path<u64>,
    body: web::Json<CreateCommentRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_comments::dsl as mrc;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    let request_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let request: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(request_id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;

    let user_id = auth.user_id()?;

    let can_comment = auth.has_any_role(&["Admin", "Manager"])
        || request.created_by == user_id
        || request.assigned_to == Some(user_id);

    if !can_comment {
        return Err(AppError::Forbidden);
    }

    let new_comment = NewMaintenanceRequestComment {
        request_id,
        user_id,
        comment_text: body.comment_text.clone(),
    };

    diesel::insert_into(mrc::maintenance_request_comments)
        .values(&new_comment)
        .execute(&mut conn)?;

    let comment: (
        u64,
        u64,
        u64,
        String,
        bool,
        Option<chrono::NaiveDateTime>,
        Option<chrono::NaiveDateTime>,
        String,
    ) = mrc::maintenance_request_comments
        .inner_join(u::users)
        .filter(mrc::request_id.eq(request_id))
        .filter(mrc::user_id.eq(user_id))
        .select((
            mrc::id,
            mrc::request_id,
            mrc::user_id,
            mrc::comment_text,
            mrc::is_deleted,
            mrc::created_at,
            mrc::updated_at,
            u::name,
        ))
        .order_by(mrc::created_at.desc())
        .first(&mut conn)?;

    let enriched = MaintenanceRequestCommentWithUser {
        id: comment.0,
        request_id: comment.1,
        user_id: comment.2,
        user_name: comment.7,
        comment_text: comment.3,
        is_deleted: comment.4,
        created_at: comment.5,
        updated_at: comment.6,
    };

    Ok(HttpResponse::Created().json(enriched))
}

/// Delete (soft-delete) a comment
///
/// Soft-deletes a comment by setting is_deleted to true.
/// Users can delete their own comments, Admin/Manager can delete any comment.
#[utoipa::path(
    delete,
    path = "/api/v1/requests/{id}/comments/{comment_id}",
    params(
        ("id" = u64, Path, description = "Maintenance request ID"),
        ("comment_id" = u64, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment deleted successfully"),
        (status = 403, description = "Forbidden - not authorized to delete this comment"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn delete_comment(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_comments::dsl as mrc;

    let (_request_id, comment_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let comment: MaintenanceRequestComment = mrc::maintenance_request_comments
        .filter(mrc::id.eq(comment_id))
        .select(MaintenanceRequestComment::as_select())
        .first(&mut conn)?;

    let user_id = auth.user_id()?;

    let can_delete = auth.has_any_role(&["Admin", "Manager"]) || comment.user_id == user_id;

    if !can_delete {
        return Err(AppError::Forbidden);
    }

    diesel::update(mrc::maintenance_request_comments.filter(mrc::id.eq(comment_id)))
        .set(mrc::is_deleted.eq(true))
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Comment deleted successfully"})))
}
