use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::{MaintenanceRequest, NewMaintenanceRequest, MaintenanceRequestHistory};
mod attachments;

/// List maintenance requests (admin/manager see all; others see only those they created for now)
pub async fn list_requests(auth: AuthContext, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let base = mr::maintenance_requests.into_boxed();
    let filtered = if auth.has_any_role(&["Admin", "Manager"]) {
        base
    } else {
        // show requests created by user or assigned to user
        base.filter(mr::created_by.eq(user_id).or(mr::assigned_to.eq(Some(user_id))))
    };
    let list = filtered.select(MaintenanceRequest::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

/// Create a maintenance request (Homeowner, Renter, Admin, Manager)
pub async fn create_request(auth: AuthContext, pool: web::Data<DbPool>, payload: web::Json<NewMaintenanceRequest>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    if !auth.has_any_role(&["Homeowner", "Renter", "Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let new = payload.into_inner();
    // default status Open
    diesel::insert_into(mr::maintenance_requests)
        .values((
            mr::apartment_id.eq(new.apartment_id),
            mr::created_by.eq(auth.claims.sub.parse::<u64>().unwrap_or(0)),
            mr::request_type.eq(new.request_type),
            mr::priority.eq(new.priority),
            mr::title.eq(new.title),
            mr::description.eq(new.description),
            mr::status.eq("Open"),
        ))
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

#[derive(serde::Deserialize)]
pub struct StatusUpdatePayload { pub status: String, pub note: Option<String> }

/// Update request status (Admin/Manager)
pub async fn update_status(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>, payload: web::Json<StatusUpdatePayload>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::maintenance_request_history::dsl as hist;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let current: MaintenanceRequest = mr::maintenance_requests.filter(mr::id.eq(id)).select(MaintenanceRequest::as_select()).first(&mut conn)?;
    let new_status = payload.status.clone();
    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id))).set((mr::status.eq(&new_status), mr::resolution_notes.eq(payload.note.clone()))) .execute(&mut conn)?;
    diesel::insert_into(hist::maintenance_request_history)
        .values((
            hist::request_id.eq(id),
            hist::from_status.eq(current.status),
            hist::to_status.eq(new_status),
            hist::note.eq(payload.note.clone()),
            hist::changed_by.eq(auth.claims.sub.parse::<u64>().unwrap_or(0)),
        ))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

/// List history entries for a request
pub async fn list_history(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    let request_id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    // simple permission check: allow if admin/manager or creator
    let req: MaintenanceRequest = mr::maintenance_requests.filter(mr::id.eq(request_id)).select(MaintenanceRequest::as_select()).first(&mut conn)?;
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    if !(auth.has_any_role(&["Admin", "Manager"]) || req.created_by == user_id) { return Err(AppError::Forbidden); }
    let entries = hist::maintenance_request_history.filter(hist::request_id.eq(request_id)).select(MaintenanceRequestHistory::as_select()).order(hist::changed_at.asc()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(entries))
}

/// Assign a maintenance request to a user (Admin/Manager only)
#[derive(serde::Deserialize)]
pub struct AssignPayload { pub user_id: u64 }

pub async fn assign_request(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>, payload: web::Json<AssignPayload>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let target_user = payload.user_id;
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    // verify user exists
    let exists: Result<u64, _> = u::users.filter(u::id.eq(target_user)).select(u::id).first(&mut conn);
    if exists.is_err() { return Err(AppError::BadRequest("user_not_found".into())); }
    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
        .set(mr::assigned_to.eq(Some(target_user)))
        .execute(&mut conn)?;
    let updated: MaintenanceRequest = mr::maintenance_requests.filter(mr::id.eq(id)).select(MaintenanceRequest::as_select()).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(updated))
}

/// Unassign a maintenance request (Admin/Manager only)
pub async fn unassign_request(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
        .set(mr::assigned_to.eq::<Option<u64>>(None))
        .execute(&mut conn)?;
    let updated: MaintenanceRequest = mr::maintenance_requests.filter(mr::id.eq(id)).select(MaintenanceRequest::as_select()).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(updated))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/requests", web::get().to(list_requests))
        .route("/requests", web::post().to(create_request))
        .route("/requests/{id}/status", web::put().to(update_status))
        .route("/requests/{id}/history", web::get().to(list_history))
        .route("/requests/{id}/assign", web::put().to(assign_request))
        .route("/requests/{id}/assign", web::delete().to(unassign_request))
        // attachment endpoints
        .route("/requests/{id}/attachments", web::post().to(attachments::upload_attachment))
        .route("/requests/{id}/attachments", web::get().to(attachments::list_attachments))
        .route("/requests/{id}/attachments/deleted", web::get().to(attachments::list_deleted_attachments))
        .route("/requests/{id}/attachments/{att_id}", web::get().to(attachments::get_attachment_metadata))
        .route("/requests/{id}/attachments/{att_id}/download", web::get().to(attachments::download_attachment))
        .route("/requests/{id}/attachments/{att_id}", web::delete().to(attachments::delete_attachment))
        .route("/requests/{id}/attachments/{att_id}/restore", web::post().to(attachments::restore_attachment));
}
