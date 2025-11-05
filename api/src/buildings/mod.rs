use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{Building, NewBuilding};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

pub async fn list_buildings(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = buildings.filter(is_deleted.eq(false)).select(Building::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

pub async fn create_building(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    item: web::Json<NewBuilding>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl as b_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::insert_into(b_dsl::buildings)
        .values(&*item)
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

pub async fn delete_building(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl as b_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(b_dsl::buildings.filter(b_dsl::id.eq(id))).set(b_dsl::is_deleted.eq(true)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_deleted_buildings(auth: AuthContext, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl::*;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = buildings.filter(is_deleted.eq(true)).select(Building::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

pub async fn restore_building(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl as b_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(b_dsl::buildings.filter(b_dsl::id.eq(id))).set(b_dsl::is_deleted.eq(false)).execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/buildings", web::get().to(list_buildings))
        .route("/buildings", web::post().to(create_building))
        .route("/buildings/deleted", web::get().to(list_deleted_buildings))
        .route("/buildings/{id}/restore", web::post().to(restore_building))
        .route("/buildings/{id}", web::delete().to(delete_building));
}
