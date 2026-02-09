use super::helpers::{conn, enrich, enrich_one, render_markdown};
use super::types::{CreateAnnouncementRequest, UpdateAnnouncementRequest};
use crate::auth::{error::AppError, extractor::AuthContext};
use crate::db::DbPool;
use crate::models::{Announcement, NewAnnouncement};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::schema::announcements;
use actix_web::{HttpResponse, web};
use chrono::Utc;
use diesel::prelude::*;
use utoipa;

/// List public announcements
///
/// Returns all published, non-expired, non-deleted public announcements.
/// No authentication required. Announcements are ordered by pinned status, then creation date.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/public",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of public announcements", body = PaginatedResponse<super::types::AnnouncementOut>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements"
)]
pub async fn list_public(
    pool: web::Data<DbPool>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    let mut c = conn(&pool)?;
    let now = Utc::now().naive_utc();

    let base_filter = a::announcements
        .filter(a::is_deleted.eq(false))
        .filter(a::publish_at.is_not_null().and(a::publish_at.le(now)))
        .filter(a::expire_at.is_null().or(a::expire_at.gt(now)))
        .filter(a::public.eq(true));

    let total = base_filter.count().get_result::<i64>(&mut c)?;

    let items = base_filter
        .order((a::pinned.desc(), a::created_at.desc()))
        .limit(query.limit())
        .offset(query.offset())
        .load::<Announcement>(&mut c)?;
    let enriched = enrich(items, &mut c)?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(enriched, total, &query)))
}

/// List announcements for authenticated users
///
/// Returns announcements based on user role:
/// - Admin/Manager: See all announcements including drafts and scheduled
/// - Others: See public announcements and role-specific private announcements
#[utoipa::path(
    get,
    path = "/api/v1/announcements",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of announcements", body = PaginatedResponse<super::types::AnnouncementOut>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn list_auth(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    use crate::auth::get_user_building_ids;
    use announcements::dsl as a;

    let mut c = conn(&pool)?;
    let now = Utc::now().naive_utc();
    let roles = auth.claims.roles.clone();
    let user_id = auth.user_id()?;
    let is_admin = auth.has_any_role(&["Admin"]);
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);

    let building_ids = if !is_admin {
        get_user_building_ids(user_id, is_admin, &mut c)?
    } else {
        None
    };

    let mut db_query = a::announcements
        .filter(a::is_deleted.eq(false))
        .into_boxed();

    if !is_manager {
        db_query = db_query
            .filter(a::publish_at.is_null().or(a::publish_at.le(now)))
            .filter(a::expire_at.is_null().or(a::expire_at.gt(now)));
    }

    if let Some(ref ids) = building_ids {
        db_query = db_query.filter(a::building_id.eq_any(ids).or(a::building_id.is_null()));
    }

    let items = db_query
        .order((a::pinned.desc(), a::created_at.desc()))
        .load::<Announcement>(&mut c)?;

    let filtered: Vec<Announcement> = items
        .into_iter()
        .filter(|ann| {
            if ann.public {
                return true;
            }
            if is_manager {
                return true;
            }
            match &ann.roles_csv {
                Some(csv) => {
                    let needed: Vec<&str> = csv
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();
                    roles.iter().any(|r| needed.iter().any(|n| r == n))
                }
                None => true,
            }
        })
        .collect();

    let total = filtered.len() as i64;
    let offset = query.offset() as usize;
    let limit = query.limit() as usize;
    let page_items: Vec<Announcement> = filtered.into_iter().skip(offset).take(limit).collect();
    let enriched = enrich(page_items, &mut c)?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(enriched, total, &query)))
}

/// Get a single announcement
///
/// Returns details of a specific announcement. Public announcements can be viewed by anyone.
/// Private announcements require authentication and appropriate role. Admin/Manager can view drafts and scheduled posts.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/{id}",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 200, description = "Announcement details", body = super::types::AnnouncementOut),
        (status = 401, description = "Unauthorized - private announcement requires authentication"),
        (status = 403, description = "Forbidden - user doesn't have required role"),
        (status = 404, description = "Announcement not found or deleted"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements"
)]
pub async fn get_one(
    pool: web::Data<DbPool>,
    auth_opt: Option<AuthContext>,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    let now = Utc::now().naive_utc();
    if ann.is_deleted {
        return Err(AppError::NotFound);
    }
    let is_manager = auth_opt
        .as_ref()
        .map(|a| a.has_any_role(&["Admin", "Manager"]).then_some(()))
        .is_some();
    if !ann.public {
        if let Some(auth) = &auth_opt {
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
        } else {
            return Err(AppError::Unauthorized);
        }
    }
    if !is_manager {
        if ann.publish_at.map(|p| p > now).unwrap_or(false) {
            return Err(AppError::NotPublished);
        }
        if ann.expire_at.map(|e| e <= now).unwrap_or(false) {
            return Err(AppError::Expired);
        }
    }
    Ok(HttpResponse::Ok().json(enrich_one(ann, &mut c)?))
}

/// Create an announcement
///
/// Creates a new announcement with markdown content (automatically rendered to HTML).
/// Requires Admin or Manager role. Supports drafts (publish_at = NULL), scheduling, and expiration.
#[utoipa::path(
    post,
    path = "/api/v1/announcements",
    request_body = CreateAnnouncementRequest,
    responses(
        (status = 201, description = "Announcement created successfully", body = super::types::AnnouncementOut),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    body: web::Json<CreateAnnouncementRequest>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let mut c = conn(&pool)?;
    let html = render_markdown(&body.body_md);
    let new = NewAnnouncement {
        title: body.title.clone(),
        body_md: body.body_md.clone(),
        body_html: html,
        author_id: auth
            .claims
            .sub
            .parse::<u64>()
            .map_err(|_| AppError::BadRequest("invalid_sub".into()))?,
        public: body.public,
        pinned: body.pinned,
        roles_csv: body.roles_csv.clone(),
        building_id: body.building_id,
        apartment_id: body.apartment_id,
        comments_enabled: body.comments_enabled,
        publish_at: body.publish_at,
        expire_at: body.expire_at,
    };
    diesel::insert_into(a::announcements)
        .values(&new)
        .execute(&mut c)?;
    let inserted = a::announcements
        .order(a::id.desc())
        .first::<Announcement>(&mut c)?;
    Ok(HttpResponse::Created().json(enrich_one(inserted, &mut c)?))
}

/// Update an announcement
///
/// Updates announcement fields. Accessible by Admin, Manager, or the announcement author.
/// If body_md is updated, body_html is automatically regenerated.
#[utoipa::path(
    put,
    path = "/api/v1/announcements/{id}",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    request_body = UpdateAnnouncementRequest,
    responses(
        (status = 200, description = "Announcement updated successfully", body = super::types::AnnouncementOut),
        (status = 403, description = "Forbidden - requires Admin, Manager, or author"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
    body: web::Json<UpdateAnnouncementRequest>,
) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    let is_author = ann.author_id.to_string() == auth.claims.sub;
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);
    if !(is_manager || is_author) {
        return Err(AppError::Forbidden);
    }
    #[derive(diesel::AsChangeset, Default)]
    #[diesel(table_name = announcements)]
    struct AnnChanges {
        title: Option<String>,
        body_md: Option<String>,
        body_html: Option<String>,
        public: Option<bool>,
        pinned: Option<bool>,
        roles_csv: Option<Option<String>>,
        building_id: Option<Option<u64>>,
        apartment_id: Option<Option<u64>>,
        comments_enabled: Option<bool>,
        publish_at: Option<Option<chrono::NaiveDateTime>>,
        expire_at: Option<Option<chrono::NaiveDateTime>>,
    }
    let mut ch = AnnChanges::default();
    if let Some(v) = &body.title {
        ch.title = Some(v.clone());
    }
    if let Some(v) = &body.body_md {
        ch.body_md = Some(v.clone());
        ch.body_html = Some(render_markdown(v));
    }
    if let Some(v) = body.public {
        ch.public = Some(v);
    }
    if let Some(v) = body.pinned {
        ch.pinned = Some(v);
    }
    if let Some(v) = &body.roles_csv {
        ch.roles_csv = Some(v.clone());
    }
    if let Some(v) = &body.building_id {
        ch.building_id = Some(*v);
    }
    if let Some(v) = &body.apartment_id {
        ch.apartment_id = Some(*v);
    }
    if let Some(v) = body.comments_enabled {
        ch.comments_enabled = Some(v);
    }
    if let Some(v) = &body.publish_at {
        ch.publish_at = Some(*v);
    }
    if let Some(v) = &body.expire_at {
        ch.expire_at = Some(*v);
    }
    diesel::update(a::announcements.filter(a::id.eq(id)))
        .set(&ch)
        .execute(&mut c)?;
    let updated = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(enrich_one(updated, &mut c)?))
}

/// Soft-delete an announcement
///
/// Marks an announcement as deleted (soft-delete). Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/announcements/{id}",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 204, description = "Announcement deleted successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn delete_soft(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    diesel::update(a::announcements.filter(a::id.eq(id)))
        .set(a::is_deleted.eq(true))
        .execute(&mut c)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Restore a soft-deleted announcement
///
/// Restores an announcement that was previously soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/announcements/{id}/restore",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 200, description = "Announcement restored successfully", body = Announcement),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn restore(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    diesel::update(a::announcements.filter(a::id.eq(id)))
        .set(a::is_deleted.eq(false))
        .execute(&mut c)?;
    let ann = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(ann))
}

/// Toggle pin status of an announcement
///
/// Pins or unpins an announcement. Pinned announcements appear first in lists.
/// Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/announcements/{id}/pin",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 200, description = "Pin status toggled successfully", body = super::types::AnnouncementOut),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn toggle_pin(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    diesel::update(a::announcements.filter(a::id.eq(id)))
        .set(a::pinned.eq(!ann.pinned))
        .execute(&mut c)?;
    let updated = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(enrich_one(updated, &mut c)?))
}

/// List soft-deleted announcements
///
/// Returns all announcements that have been soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/deleted",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of deleted announcements", body = PaginatedResponse<super::types::AnnouncementOut>),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn list_deleted(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let mut c = conn(&pool)?;

    let total = a::announcements
        .filter(a::is_deleted.eq(true))
        .count()
        .get_result::<i64>(&mut c)?;

    let items = a::announcements
        .filter(a::is_deleted.eq(true))
        .order(a::created_at.desc())
        .limit(query.limit())
        .offset(query.offset())
        .load::<Announcement>(&mut c)?;
    let enriched = enrich(items, &mut c)?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(enriched, total, &query)))
}

/// Permanently delete an announcement (purge)
///
/// Permanently removes a soft-deleted announcement from the database.
/// This also deletes all associated comments. Requires Admin or Manager role.
/// The announcement must be soft-deleted first before it can be purged.
#[utoipa::path(
    delete,
    path = "/api/v1/announcements/{id}/purge",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 204, description = "Announcement permanently deleted"),
        (status = 400, description = "Bad request - announcement not soft-deleted"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn purge(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use crate::schema::announcements::dsl as a;
    use crate::schema::announcements_comments::dsl as cmt;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    match a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)
    {
        Ok(ann) => {
            if !ann.is_deleted {
                return Err(AppError::BadRequest("not_soft_deleted".into()));
            }
        }
        Err(_) => return Err(AppError::NotFound),
    }
    diesel::delete(cmt::announcements_comments.filter(cmt::announcement_id.eq(id)))
        .execute(&mut c)?;
    let affected = diesel::delete(a::announcements.filter(a::id.eq(id))).execute(&mut c)?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(HttpResponse::NoContent().finish())
}

/// Publish an announcement immediately
///
/// Sets the publish_at timestamp to now, making a draft or scheduled announcement
/// immediately visible. Accessible by Admin, Manager, or the announcement author.
#[utoipa::path(
    post,
    path = "/api/v1/announcements/{id}/publish",
    params(
        ("id" = u64, Path, description = "Announcement ID")
    ),
    responses(
        (status = 200, description = "Announcement published successfully", body = super::types::AnnouncementOut),
        (status = 403, description = "Forbidden - requires Admin, Manager, or author"),
        (status = 404, description = "Announcement not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn publish_now(
    pool: web::Data<DbPool>,
    auth: AuthContext,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    let ann = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    let is_author = ann.author_id.to_string() == auth.claims.sub;
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);
    if !(is_manager || is_author) {
        return Err(AppError::Forbidden);
    }
    let now = Utc::now().naive_utc();
    if ann.publish_at.map(|p| p <= now).unwrap_or(false) {
        return Ok(HttpResponse::Ok().json(ann));
    }
    diesel::update(a::announcements.filter(a::id.eq(id)))
        .set(a::publish_at.eq(now))
        .execute(&mut c)?;
    let updated = a::announcements
        .filter(a::id.eq(id))
        .first::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(enrich_one(updated, &mut c)?))
}
