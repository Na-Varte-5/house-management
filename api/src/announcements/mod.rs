use crate::auth::{error::AppError, extractor::AuthContext};
use crate::db::DbPool;
use crate::models::{Announcement, AnnouncementComment, NewAnnouncement, NewAnnouncementComment};
use crate::schema::users::dsl as u;
use crate::schema::{announcements, announcements_comments};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use utoipa;

fn render_markdown(md: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    // Ammonia's clean now returns a Document; convert it to String.
    ammonia::Builder::default().clean(&html_output).to_string()
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub body_md: String,
    pub public: bool,
    pub pinned: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub apartment_id: Option<u64>,
    pub comments_enabled: bool,
    pub publish_at: Option<chrono::NaiveDateTime>,
    pub expire_at: Option<chrono::NaiveDateTime>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateAnnouncementRequest {
    pub title: Option<String>,
    pub body_md: Option<String>,
    pub public: Option<bool>,
    pub pinned: Option<bool>,
    pub roles_csv: Option<Option<String>>, // double option to allow clearing
    pub building_id: Option<Option<u64>>,
    pub apartment_id: Option<Option<u64>>,
    pub comments_enabled: Option<bool>,
    pub publish_at: Option<Option<chrono::NaiveDateTime>>,
    pub expire_at: Option<Option<chrono::NaiveDateTime>>,
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct CommentsQuery {
    pub include_deleted: Option<bool>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCommentRequest {
    pub body_md: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AnnouncementOut {
    id: u64,
    title: String,
    body_md: String,
    body_html: String,
    author_id: u64,
    author_name: String,
    public: bool,
    pinned: bool,
    roles_csv: Option<String>,
    building_id: Option<u64>,
    building_address: Option<String>,
    apartment_id: Option<u64>,
    apartment_number: Option<String>,
    comments_enabled: bool,
    publish_at: Option<chrono::NaiveDateTime>,
    expire_at: Option<chrono::NaiveDateTime>,
    is_deleted: bool,
    created_at: Option<chrono::NaiveDateTime>,
    updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CommentOut {
    id: u64,
    announcement_id: u64,
    user_id: u64,
    user_name: String,
    body_md: String,
    body_html: String,
    is_deleted: bool,
    created_at: Option<chrono::NaiveDateTime>,
}

fn enrich(
    mut anns: Vec<Announcement>,
    conn: &mut diesel::MysqlConnection,
) -> Result<Vec<AnnouncementOut>, AppError> {
    use crate::schema::apartments::dsl as ap;
    use crate::schema::buildings::dsl as b;
    let ids: Vec<u64> = anns.iter().map(|a| a.author_id).collect();
    let users = u::users
        .filter(u::id.eq_any(&ids))
        .load::<crate::models::User>(conn)?;
    let mut user_map: HashMap<u64, String> = HashMap::new();
    for usr in users {
        user_map.insert(usr.id, usr.name);
    }
    // building addresses
    let b_ids: Vec<u64> = anns
        .iter()
        .filter_map(|a| a.building_id)
        .collect::<Vec<_>>();
    let b_ids_uniq: Vec<u64> = {
        let mut tmp = b_ids.clone();
        tmp.sort_unstable();
        std::mem::take(&mut tmp)
    }; // may contain dups; sorting then collect keeps order but okay
    let building_rows: Vec<(u64, String)> = if b_ids_uniq.is_empty() {
        vec![]
    } else {
        b::buildings
            .filter(b::id.eq_any(&b_ids_uniq))
            .select((b::id, b::address))
            .load(conn)?
    };
    let mut building_map: HashMap<u64, String> = HashMap::new();
    for (bid, addr) in building_rows {
        building_map.insert(bid, addr);
    }
    // apartment numbers
    let ap_ids: Vec<u64> = anns
        .iter()
        .filter_map(|a| a.apartment_id)
        .collect::<Vec<_>>();
    let ap_ids_uniq: Vec<u64> = {
        let mut tmp = ap_ids.clone();
        tmp.sort_unstable();
        std::mem::take(&mut tmp)
    };
    let apartment_rows: Vec<(u64, String)> = if ap_ids_uniq.is_empty() {
        vec![]
    } else {
        ap::apartments
            .filter(ap::id.eq_any(&ap_ids_uniq))
            .select((ap::id, ap::number))
            .load(conn)?
    };
    let mut apartment_map: HashMap<u64, String> = HashMap::new();
    for (aid, num) in apartment_rows {
        apartment_map.insert(aid, num);
    }
    Ok(anns
        .drain(..)
        .map(|a| AnnouncementOut {
            author_name: user_map
                .get(&a.author_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".into()),
            id: a.id,
            title: a.title,
            body_md: a.body_md,
            body_html: a.body_html,
            author_id: a.author_id,
            public: a.public,
            pinned: a.pinned,
            roles_csv: a.roles_csv,
            building_id: a.building_id,
            building_address: a
                .building_id
                .and_then(|bid| building_map.get(&bid).cloned()),
            apartment_id: a.apartment_id,
            apartment_number: a
                .apartment_id
                .and_then(|aid| apartment_map.get(&aid).cloned()),
            comments_enabled: a.comments_enabled,
            publish_at: a.publish_at,
            expire_at: a.expire_at,
            is_deleted: a.is_deleted,
            created_at: a.created_at,
            updated_at: a.updated_at,
        })
        .collect())
}

fn enrich_one(
    a: Announcement,
    conn: &mut diesel::MysqlConnection,
) -> Result<AnnouncementOut, AppError> {
    use crate::schema::apartments::dsl as ap;
    use crate::schema::buildings::dsl as b;
    let name = u::users
        .filter(u::id.eq(a.author_id))
        .select(u::name)
        .first::<String>(conn)
        .unwrap_or_else(|_| "Unknown".into());
    let building_address = match a.building_id {
        Some(bid) => b::buildings
            .filter(b::id.eq(bid))
            .select(b::address)
            .first::<String>(conn)
            .ok(),
        None => None,
    };
    let apartment_number = match a.apartment_id {
        Some(aid) => ap::apartments
            .filter(ap::id.eq(aid))
            .select(ap::number)
            .first::<String>(conn)
            .ok(),
        None => None,
    };
    Ok(AnnouncementOut {
        author_name: name,
        id: a.id,
        title: a.title,
        body_md: a.body_md,
        body_html: a.body_html,
        author_id: a.author_id,
        public: a.public,
        pinned: a.pinned,
        roles_csv: a.roles_csv,
        building_id: a.building_id,
        building_address,
        apartment_id: a.apartment_id,
        apartment_number,
        comments_enabled: a.comments_enabled,
        publish_at: a.publish_at,
        expire_at: a.expire_at,
        is_deleted: a.is_deleted,
        created_at: a.created_at,
        updated_at: a.updated_at,
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/announcements")
            .route("/public", web::get().to(list_public))
            .route("/deleted", web::get().to(list_deleted))
            .route("", web::get().to(list_auth))
            .route("", web::post().to(create))
            .route("/{id}", web::get().to(get_one))
            .route("/{id}", web::put().to(update))
            .route("/{id}", web::delete().to(delete_soft))
            .route("/{id}/restore", web::post().to(restore))
            .route("/{id}/pin", web::post().to(toggle_pin))
            .route("/{id}/comments", web::get().to(list_comments))
            .route("/{id}/comments", web::post().to(create_comment))
            .route("/comments/{comment_id}", web::delete().to(delete_comment))
            .route(
                "/comments/{comment_id}/restore",
                web::post().to(restore_comment),
            )
            .route(
                "/comments/{comment_id}/purge",
                web::delete().to(purge_comment),
            )
            .route("/{id}/purge", web::delete().to(purge))
            .route("/{id}/publish", web::post().to(publish_now)),
    );
}

fn conn(
    pool: &web::Data<DbPool>,
) -> Result<
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>,
    AppError,
> {
    pool.get().map_err(|_| AppError::Internal("db_pool".into()))
}

/// List public announcements
///
/// Returns all published, non-expired, non-deleted public announcements.
/// No authentication required. Announcements are ordered by pinned status, then creation date.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/public",
    responses(
        (status = 200, description = "List of public announcements", body = Vec<AnnouncementOut>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements"
)]
pub async fn list_public(pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use announcements::dsl as a;
    let mut c = conn(&pool)?;
    let now = Utc::now().naive_utc();
    let items = a::announcements
        .filter(a::is_deleted.eq(false))
        // draft: publish_at IS NULL => excluded from public list
        .filter(a::publish_at.is_not_null().and(a::publish_at.le(now)))
        .filter(a::expire_at.is_null().or(a::expire_at.gt(now)))
        .filter(a::public.eq(true))
        .order((a::pinned.desc(), a::created_at.desc()))
        .load::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(enrich(items, &mut c)?))
}

/// List announcements for authenticated users
///
/// Returns announcements based on user role:
/// - Admin/Manager: See all announcements including drafts and scheduled
/// - Others: See public announcements and role-specific private announcements
#[utoipa::path(
    get,
    path = "/api/v1/announcements",
    responses(
        (status = 200, description = "List of announcements", body = Vec<AnnouncementOut>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn list_auth(
    pool: web::Data<DbPool>,
    auth: AuthContext,
) -> Result<HttpResponse, AppError> {
    use crate::auth::get_user_building_ids;
    use announcements::dsl as a;

    let mut c = conn(&pool)?;
    let now = Utc::now().naive_utc();
    let roles = auth.claims.roles.clone();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin = auth.has_any_role(&["Admin"]);
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);

    // Get building access for non-admin users
    let building_ids = if !is_admin {
        get_user_building_ids(user_id, is_admin, &mut c)?
    } else {
        None
    };

    // Build query conditionally to avoid mixing runtime bools into Diesel expression trees.
    let mut query = a::announcements
        .filter(a::is_deleted.eq(false))
        .into_boxed();

    if !is_manager {
        query = query
            .filter(a::publish_at.is_null().or(a::publish_at.le(now)))
            .filter(a::expire_at.is_null().or(a::expire_at.gt(now)));
    }

    // Building-scoped filtering: non-Admin users see only announcements from accessible buildings OR global announcements
    if let Some(ref ids) = building_ids {
        query = query.filter(a::building_id.eq_any(ids).or(a::building_id.is_null()));
    }

    let items = query
        .order((a::pinned.desc(), a::created_at.desc()))
        .load::<Announcement>(&mut c)?;

    // filter by roles_csv client side (CSV intersection) unless public or manager
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
    Ok(HttpResponse::Ok().json(enrich(filtered, &mut c)?))
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
        (status = 200, description = "Announcement details", body = AnnouncementOut),
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
    // scheduling checks for public/unprivileged access
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
        (status = 201, description = "Announcement created successfully", body = AnnouncementOut),
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
        (status = 200, description = "Announcement updated successfully", body = AnnouncementOut),
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
        (status = 200, description = "Pin status toggled successfully", body = AnnouncementOut),
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
    // Visibility for non-public announcements
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
    } else {
        // public announcement scheduling constraints for non-manager / anonymous
        if !is_manager {
            if ann.publish_at.map(|p| p > now).unwrap_or(true) {
                return Err(AppError::NotPublished);
            }
            if ann.expire_at.map(|e| e <= now).unwrap_or(false) {
                return Err(AppError::Expired);
            }
        }
    }
    // include_deleted only for managers
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
    // enrich user names
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
    if !ann.public && !auth.has_any_role(&["Admin", "Manager"]) && let Some(csv) = &ann.roles_csv {
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

/// List soft-deleted announcements
///
/// Returns all announcements that have been soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/announcements/deleted",
    responses(
        (status = 200, description = "List of deleted announcements", body = Vec<AnnouncementOut>),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Announcements",
    security(("bearer_auth" = []))
)]
pub async fn list_deleted(
    pool: web::Data<DbPool>,
    auth: AuthContext,
) -> Result<HttpResponse, AppError> {
    auth.require_roles(&["Admin", "Manager"])?;
    use announcements::dsl as a;
    let mut c = conn(&pool)?;
    let items = a::announcements
        .filter(a::is_deleted.eq(true))
        .order(a::created_at.desc())
        .load::<Announcement>(&mut c)?;
    Ok(HttpResponse::Ok().json(enrich(items, &mut c)?))
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
    use announcements::dsl as a;
    use announcements_comments::dsl as cmt;
    let id = path.into_inner();
    let mut c = conn(&pool)?;
    // Ensure exists and is soft-deleted
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
    // Remove dependent comments first
    diesel::delete(cmt::announcements_comments.filter(cmt::announcement_id.eq(id)))
        .execute(&mut c)?;
    // Delete announcement
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
        (status = 200, description = "Announcement published successfully", body = AnnouncementOut),
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
    // Only set if draft (publish_at NULL) or scheduled future
    if ann.publish_at.map(|p| p <= now).unwrap_or(false) {
        // Already published
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
    // ensure exists and is soft-deleted
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
