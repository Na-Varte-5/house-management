use actix_web::{web, HttpResponse};
use crate::auth::{AuthContext, AppError};
use crate::db::DbPool;
use crate::config::AppConfig;
use diesel::prelude::*;
use crate::models::MaintenanceRequestAttachment;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use uuid::Uuid;
use std::io::Write;
use std::fs;
use actix_web::http::header::{CONTENT_TYPE, CONTENT_DISPOSITION};
use crate::models::MaintenanceRequest; // for RBAC checks
use crate::schema::{maintenance_requests as mr, apartment_owners as ao};
use diesel::mysql::MysqlConnection;
use utoipa;


/// Upload a maintenance request attachment
///
/// Uploads a file attachment to a maintenance request using multipart/form-data.
/// File size limit and MIME type restrictions apply (configured in AppConfig).
/// Accessible by Admin, Manager, request creator, or assigned user.
#[utoipa::path(
    post,
    path = "/api/v1/requests/{id}/attachments",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Attachment uploaded successfully"),
        (status = 400, description = "Bad request - no file or invalid MIME type"),
        (status = 403, description = "Forbidden - cannot modify this request"),
        (status = 413, description = "File too large"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn upload_attachment(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>, cfg: web::Data<AppConfig>, mut payload: Multipart) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let request_id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let owns = user_owns_apartment(auth.claims.sub.parse().unwrap_or(0), req.apartment_id, &mut conn)?; // ownership for view not needed for upload but reused
    let perms = compute_perms(&auth, &req, owns, auth.claims.sub.parse().unwrap_or(0));
    if !perms.can_modify { return Err(AppError::Forbidden); }
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut original_filename = None;
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::Internal(format!("multipart: {}", e)))?;
        let name = field.name().to_string();
        if name != "file" { continue; }
        if let Some(fname) = field.content_disposition().get_filename() { original_filename = Some(fname.to_string()); }
        while let Some(chunk_res) = field.next().await {
            let chunk = chunk_res.map_err(|e| AppError::Internal(format!("chunk: {}", e)))?;
            file_bytes.extend_from_slice(&chunk);
            if file_bytes.len() as u64 > cfg.max_attachment_size_bytes { return Err(AppError::AttachmentTooLarge); }
        }
        break; // only first file
    }
    let original_filename = original_filename.unwrap_or_else(|| "upload.bin".into());
    if file_bytes.is_empty() { return Err(AppError::BadRequest("no_file".into())); }
    let detected = infer::get(&file_bytes).map(|t| t.mime_type());
    let mime = detected.unwrap_or("application/octet-stream");
    if !cfg.allowed_mime_types.iter().any(|m| m == mime) { return Err(AppError::InvalidMimeType); }
    let stored_filename = format!("{}", Uuid::new_v4());
    let dir_path = std::path::Path::new(&cfg.attachments_base_path).join(request_id.to_string());
    fs::create_dir_all(&dir_path).map_err(|e| AppError::Internal(format!("fs_create_dir: {}", e)))?;
    let tmp_path = dir_path.join(format!("{}.tmp", stored_filename));
    let final_path = dir_path.join(&stored_filename);
    {
        let mut f = std::fs::File::create(&tmp_path).map_err(|e| AppError::Internal(format!("file_create: {}", e)))?;
        f.write_all(&file_bytes).map_err(|e| AppError::Internal(format!("file_write: {}", e)))?;
    }
    fs::rename(&tmp_path, &final_path).map_err(|e| AppError::Internal(format!("file_rename: {}", e)))?;
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::insert_into(att::maintenance_request_attachments)
        .values((
            att::request_id.eq(request_id),
            att::original_filename.eq(sanitize_filename(&original_filename)),
            att::stored_filename.eq(stored_filename.clone()),
            att::mime_type.eq(mime.to_string()),
            att::size_bytes.eq(file_bytes.len() as u64),
            att::is_deleted.eq(false),
        ))
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

fn sanitize_filename(name: &str) -> String { name.replace('/', "_").replace('\\', "_") }

fn load_request(request_id: u64, conn: &mut MysqlConnection) -> Result<MaintenanceRequest, AppError> {
    use mr::dsl as m;
    let req: MaintenanceRequest = m::maintenance_requests
        .filter(m::id.eq(request_id))
        .select(MaintenanceRequest::as_select())
        .first(conn)
        .map_err(|_| AppError::NotFound)?;
    Ok(req)
}

fn user_owns_apartment(user_id: u64, apartment_id: u64, conn: &mut MysqlConnection) -> Result<bool, AppError> {
    use ao::dsl as a;
    let exists: Result<(u64,u64), _> = a::apartment_owners
        .filter(a::apartment_id.eq(apartment_id).and(a::user_id.eq(user_id)))
        .select((a::apartment_id, a::user_id))
        .first(conn);
    Ok(exists.is_ok())
}

struct RequestPerms {
    can_view: bool,
    can_modify: bool, // upload/delete/restore
}

fn compute_perms(auth: &AuthContext, req: &MaintenanceRequest, owns: bool, user_id: u64) -> RequestPerms {
    let is_admin_mgr = auth.has_any_role(&["Admin", "Manager"]);
    let is_creator = req.created_by == user_id;
    let is_assigned = req.assigned_to.unwrap_or(0) == user_id && req.assigned_to.is_some();
    let can_view = is_admin_mgr || is_creator || is_assigned || owns;
    let can_modify = is_admin_mgr || is_creator || is_assigned;
    RequestPerms { can_view, can_modify }
}

/// List attachments (non-deleted)
///
/// Returns all non-deleted attachments for a maintenance request.
/// Accessible by Admin, Manager, request creator, assigned user, or apartment owner.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/attachments",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "List of attachments", body = Vec<MaintenanceRequestAttachment>),
        (status = 403, description = "Forbidden - cannot view this request"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn list_attachments(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let request_id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?;
    let perms = compute_perms(&auth, &req, owns, user_id);
    if !perms.can_view { return Err(AppError::Forbidden); }
    let rows = att::maintenance_request_attachments
        .filter(att::request_id.eq(request_id))
        .filter(att::is_deleted.eq(false))
        .select(MaintenanceRequestAttachment::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(rows))
}

/// List deleted attachments
///
/// Returns all soft-deleted attachments for a maintenance request.
/// Accessible by Admin, Manager, request creator, or assigned user.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/attachments/deleted",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "List of deleted attachments", body = Vec<MaintenanceRequestAttachment>),
        (status = 403, description = "Forbidden - requires modify permissions"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn list_deleted_attachments(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let request_id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?;
    let perms = compute_perms(&auth, &req, owns, user_id);
    // Deleted list restricted to modify-level (admin/manager or creator/assigned)
    if !perms.can_modify { return Err(AppError::Forbidden); }
    let rows = att::maintenance_request_attachments
        .filter(att::request_id.eq(request_id))
        .filter(att::is_deleted.eq(true))
        .select(MaintenanceRequestAttachment::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(rows))
}

/// Get attachment metadata
///
/// Returns metadata (filename, size, MIME type, etc.) for a specific attachment.
/// Accessible by Admin, Manager, request creator, assigned user, or apartment owner.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/attachments/{att_id}",
    params(
        ("id" = u64, Path, description = "Maintenance request ID"),
        ("att_id" = u64, Path, description = "Attachment ID")
    ),
    responses(
        (status = 200, description = "Attachment metadata", body = MaintenanceRequestAttachment),
        (status = 403, description = "Forbidden - cannot view this request"),
        (status = 404, description = "Attachment or request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn get_attachment_metadata(auth: AuthContext, path: web::Path<(u64,u64)>, pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let (request_id, att_id) = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?;
    let perms = compute_perms(&auth, &req, owns, user_id);
    if !perms.can_view { return Err(AppError::Forbidden); }
    let item: MaintenanceRequestAttachment = att::maintenance_request_attachments
        .filter(att::id.eq(att_id))
        .filter(att::request_id.eq(request_id))
        .select(MaintenanceRequestAttachment::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(item))
}

/// Download attachment file
///
/// Downloads the actual file content for an attachment. Returns the file with appropriate
/// Content-Type and Content-Disposition headers. Only non-deleted attachments can be downloaded.
/// Accessible by Admin, Manager, request creator, assigned user, or apartment owner.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/attachments/{att_id}/download",
    params(
        ("id" = u64, Path, description = "Maintenance request ID"),
        ("att_id" = u64, Path, description = "Attachment ID")
    ),
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 403, description = "Forbidden - cannot view this request"),
        (status = 404, description = "Attachment not found or deleted"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn download_attachment(auth: AuthContext, path: web::Path<(u64,u64)>, pool: web::Data<DbPool>, cfg: web::Data<AppConfig>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let (request_id, att_id) = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?;
    let perms = compute_perms(&auth, &req, owns, user_id);
    if !perms.can_view { return Err(AppError::Forbidden); }
    let item: MaintenanceRequestAttachment = att::maintenance_request_attachments
        .filter(att::id.eq(att_id))
        .filter(att::request_id.eq(request_id))
        .filter(att::is_deleted.eq(false))
        .select(MaintenanceRequestAttachment::as_select())
        .first(&mut conn)?;
    let file_path = std::path::Path::new(&cfg.attachments_base_path)
        .join(request_id.to_string())
        .join(&item.stored_filename);
    let data = std::fs::read(&file_path).map_err(|_| AppError::NotFound)?;
    Ok(HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, item.mime_type.clone()))
        .insert_header((CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", item.original_filename)))
        .body(data))
}

/// Soft-delete attachment
///
/// Marks an attachment as deleted (soft-delete). The file remains on disk but is hidden.
/// Accessible by Admin, Manager, request creator, or assigned user.
#[utoipa::path(
    delete,
    path = "/api/v1/requests/{id}/attachments/{att_id}",
    params(
        ("id" = u64, Path, description = "Maintenance request ID"),
        ("att_id" = u64, Path, description = "Attachment ID")
    ),
    responses(
        (status = 200, description = "Attachment deleted successfully"),
        (status = 403, description = "Forbidden - requires modify permissions"),
        (status = 404, description = "Attachment or request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn delete_attachment(auth: AuthContext, path: web::Path<(u64,u64)>, pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let (request_id, att_id) = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?; // not needed but consistent
    let perms = compute_perms(&auth, &req, owns, user_id);
    if !perms.can_modify { return Err(AppError::Forbidden); }
    diesel::update(att::maintenance_request_attachments.filter(att::id.eq(att_id)))
        .set(att::is_deleted.eq(true))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

/// Restore attachment
///
/// Restores a soft-deleted attachment, making it visible again.
/// Accessible by Admin, Manager, request creator, or assigned user.
#[utoipa::path(
    post,
    path = "/api/v1/requests/{id}/attachments/{att_id}/restore",
    params(
        ("id" = u64, Path, description = "Maintenance request ID"),
        ("att_id" = u64, Path, description = "Attachment ID")
    ),
    responses(
        (status = 200, description = "Attachment restored successfully"),
        (status = 403, description = "Forbidden - requires modify permissions"),
        (status = 404, description = "Attachment or request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn restore_attachment(auth: AuthContext, path: web::Path<(u64,u64)>, pool: web::Data<DbPool>) -> Result<HttpResponse, AppError> {
    use crate::schema::maintenance_request_attachments::dsl as att;
    let (request_id, att_id) = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let req = load_request(request_id, &mut conn)?;
    let user_id = auth.claims.sub.parse().unwrap_or(0);
    let owns = user_owns_apartment(user_id, req.apartment_id, &mut conn)?;
    let perms = compute_perms(&auth, &req, owns, user_id);
    if !perms.can_modify { return Err(AppError::Forbidden); }
    diesel::update(att::maintenance_request_attachments.filter(att::id.eq(att_id)))
        .set(att::is_deleted.eq(false))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}
