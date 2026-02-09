use super::helpers::{conn, render_markdown};
use super::types::{CommentOut, CommentsQuery, CreateCommentRequest};
use crate::auth::{error::AppError, extractor::AuthContext};
use crate::db::DbPool;
use crate::models::{Announcement, AnnouncementComment, NewAnnouncementComment};
use crate::schema::{announcements, announcements_comments};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use utoipa;

/// List comments on an announcement
///
/// Returns all comments on an announcement. For public announcements, no authentication required.
/// For private announcements, requires authentication and appropriate role.
/// Admin/Manager can use include_deleted=true query param to view deleted comments.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/{id}/comments",
    params(
        ("id" = u64, Path, description = "Announcement ID"),
        CommentsQuery
    ),
    responses(
        (status = 200, description = "List of comments", body = Vec<CommentOut>),
        (status = 401, description = "Unauthorized - private announcement requires authentication"),
        (status = 403, description = "Forbidden - user doesn't have required role"),
        (status = 404, description = "Announcement not found or comments disabled"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements"
)]
pub async fn list_comments(
    pool: web::Data<DbPool>,
    auth_opt: Option<AuthContext>,
    path: web::Path<u64>,
    q: web::Query<CommentsQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::schema::users::dsl as u;
    use announcements::dsl as a;
    use announcements_comments::dsl as cmt;
    let announcement_id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(announcement_id))
        .first::<Announcement>(&mut c)?;
    if ann.is_deleted {
        return Err(AppError::NotFound);
    }
    if !ann.comments_enabled {
        return Err(AppError::CommentsDisabled);
    }
    let now = Utc::now().naive_utc();
    let is_manager = auth_opt
        .as_ref()
        .map(|a| a.has_any_role(&["Admin", "Manager"]).then_some(()))
        .is_some();
    if !ann.public {
        let auth = auth_opt.as_ref().ok_or(AppError::Unauthorized)?;
        if !is_manager && let Some(csv) = &ann.roles_csv {
            let needed: Vec<&str> = csv
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !auth
                .claims
                .roles
                .iter()
                .any(|r| needed.iter().any(|n| r == n))
            {
                return Err(AppError::Forbidden);
            }
        }
    } else if !is_manager {
        if ann.publish_at.map(|p| p > now).unwrap_or(true) {
            return Err(AppError::NotPublished);
        }
        if ann.expire_at.map(|e| e <= now).unwrap_or(false) {
            return Err(AppError::Expired);
        }
    }
    let include_deleted = q.include_deleted.unwrap_or(false) && is_manager;
    let mut query = cmt::announcements_comments
        .filter(cmt::announcement_id.eq(announcement_id))
        .into_boxed();
    if !include_deleted {
        query = query.filter(cmt::is_deleted.eq(false));
    }
    let list = query
        .order(cmt::created_at.asc())
        .load::<AnnouncementComment>(&mut c)?;
    let user_ids: Vec<u64> = list.iter().map(|c| c.user_id).collect();
    let users = if user_ids.is_empty() {
        vec![]
    } else {
        u::users
            .filter(u::id.eq_any(&user_ids))
            .load::<crate::models::User>(&mut c)?
    };
    let mut user_map: HashMap<u64, String> = HashMap::new();
    for usr in users {
        user_map.insert(usr.id, usr.name);
    }
    let out: Vec<CommentOut> = list
        .into_iter()
        .map(|c| CommentOut {
            id: c.id,
            announcement_id: c.announcement_id,
            user_id: c.user_id,
            user_name: user_map
                .get(&c.user_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".into()),
            body_md: c.body_md,
            body_html: c.body_html,
            is_deleted: c.is_deleted,
            created_at: c.created_at,
        })
        .collect();
    Ok(HttpResponse::Ok().json(out))
}

/// Create a comment on an announcement
///
/// Adds a comment to an announcement. Markdown content is automatically rendered to HTML.
/// Requires authentication and appropriate permissions for private announcements.
#[utoipa::path(
    post,
    path = "/api/v1/announcements/{id}/comments",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created successfully", body = AnnouncementComment),
        (status = 403, description = "Forbidden - comments disabled or insufficient permissions"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn create_comment(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
    body: web::Json<CreateCommentRequest>,
) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    use announcements_comments::dsl as cmt;
    let announcement_id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(announcement_id))
        .first::<Announcement>(&mut c)?;
    if !ann.comments_enabled {
        return Err(AppError::CommentsDisabled);
    }
    if ann.is_deleted {
        return Err(AppError::NotFound);
    }
    if !ann.public
        && !auth.has_any_role(&["Admin", "Manager"])
        && let Some(csv) = &ann.roles_csv
    {
        let needed: Vec<&str> = csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !auth
            .claims
            .roles
            .iter()
            .any(|r| needed.iter().any(|n| r == n))
        {
            return Err(AppError::Forbidden);
        }
    }
    let html = render_markdown(&body.body_md);
    let new = NewAnnouncementComment {
        announcement_id,
        user_id: auth
            .claims
            .sub
            .parse::<u64>()
            .map_err(|_| AppError::BadRequest("invalid_sub".into()))?,
        body_md: body.body_md.clone(),
        body_html: html,
    };
    diesel::insert_into(cmt::announcements_comments)
        .values(&new)
        .execute(&mut c)?;
    let inserted = cmt::announcements_comments
        .order(cmt::id.desc())
        .first::<AnnouncementComment>(&mut c)?;
    Ok(HttpResponse::Created().json(inserted))
}

/// Soft-delete a comment
///
/// Marks a comment as deleted (soft-delete). Accessible by Admin, Manager, or the comment author.
#[utoipa::path(
    delete,
    path = "/api/v1/announcements/comments/{comment_id}",
    params(
        ("comment_id" = u64, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 403, description = "Forbidden - requires Admin, Manager, or author"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn delete_comment(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    use announcements_comments::dsl as cmt;
    let comment_id = path.into_inner();
    let mut c = conn(&pool)?;
    let comment = cmt::announcements_comments
        .filter(cmt::id.eq(comment_id))
        .first::<AnnouncementComment>(&mut c)?;
    let is_author = comment.user_id.to_string() == auth.claims.sub;
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);
    if !(is_author || is_manager) {
        return Err(AppError::Forbidden);
    }
    diesel::update(cmt::announcements_comments.filter(cmt::id.eq(comment_id)))
        .set(cmt::is_deleted.eq(true))
        .execute(&mut c)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Restore a soft-deleted comment
///
/// Restores a comment that was previously soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/announcements/comments/{comment_id}/restore",
    params(
        ("comment_id" = u64, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment restored successfully", body = AnnouncementComment),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn restore_comment(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements_comments::dsl as cmt;
    let comment_id = path.into_inner();
    let mut c = conn(&pool)?;
    diesel::update(cmt::announcements_comments.filter(cmt::id.eq(comment_id)))
        .set(cmt::is_deleted.eq(false))
        .execute(&mut c)?;
    let restored = cmt::announcements_comments
        .filter(cmt::id.eq(comment_id))
        .first::<AnnouncementComment>(&mut c)?;
    Ok(HttpResponse::Ok().json(restored))
}

/// Permanently delete a comment (purge)
///
/// Permanently removes a soft-deleted comment from the database. Requires Admin or Manager role.
/// The comment must be soft-deleted first before it can be purged.
#[utoipa::path(
    delete,
    path = "/api/v1/announcements/comments/{comment_id}/purge",
    params(
        ("comment_id" = u64, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment permanently deleted"),
        (status = 400, description = "Bad request - comment not soft-deleted"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Comment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn purge_comment(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements_comments::dsl as cmt;
    let comment_id = path.into_inner();
    let mut c = conn(&pool)?;
    let comment = match cmt::announcements_comments
        .filter(cmt::id.eq(comment_id))
        .first::<AnnouncementComment>(&mut c)
    {
        Ok(c) => c,
        Err(_) => return Err(AppError::NotFound),
    };
    if !comment.is_deleted {
        return Err(AppError::BadRequest("not_soft_deleted".into()));
    }
    diesel::delete(cmt::announcements_comments.filter(cmt::id.eq(comment_id))).execute(&mut c)?;
    Ok(HttpResponse::NoContent().finish())
}
