use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{Apartment, ApartmentOwner, NewApartment, PublicUser, User};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

// List all apartments
pub async fn list_apartments(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = apartments.filter(is_deleted.eq(false)).select(Apartment::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

// List apartments by building
pub async fn list_building_apartments(
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    let building = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = apartments
        .filter(building_id.eq(building).and(is_deleted.eq(false)))
        .select(Apartment::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

// Create apartment (Admin or Manager)
pub async fn create_apartment(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    item: web::Json<NewApartment>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as a_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::insert_into(a_dsl::apartments)
        .values(&*item)
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

#[derive(serde::Deserialize)]
pub struct OwnerAssignPayload {
    pub user_id: u64,
}

// List owners for apartment
pub async fn list_apartment_owners(
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::users::dsl as u;
    let apartment = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let res = ao::apartment_owners
        .inner_join(u::users.on(u::id.eq(ao::user_id)))
        .filter(ao::apartment_id.eq(apartment))
        .select(User::as_select())
        .load::<User>(&mut conn)?;
    let pub_users: Vec<PublicUser> = res.into_iter().map(PublicUser::from).collect();
    Ok(HttpResponse::Ok().json(pub_users))
}

// Assign owner
pub async fn add_apartment_owner(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<OwnerAssignPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let apartment = path.into_inner();
    let new = ApartmentOwner {
        apartment_id: apartment,
        user_id: payload.user_id,
    };
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let exists: Result<(u64, u64), _> = ao::apartment_owners
        .filter(
            ao::apartment_id
                .eq(new.apartment_id)
                .and(ao::user_id.eq(new.user_id)),
        )
        .select((ao::apartment_id, ao::user_id))
        .first(&mut conn);
    if exists.is_ok() {
        return Ok(HttpResponse::NoContent().finish());
    }
    diesel::insert_into(ao::apartment_owners)
        .values(&new)
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

// Remove owner
pub async fn remove_apartment_owner(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let (apartment, user) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::delete(
        ao::apartment_owners.filter(ao::apartment_id.eq(apartment).and(ao::user_id.eq(user))),
    )
    .execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

// Delete apartment (Admin or Manager)
pub async fn delete_apartment(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as a_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(a_dsl::apartments.filter(a_dsl::id.eq(id))).set(a_dsl::is_deleted.eq(true)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

// List deleted apartments (Admin or Manager)
pub async fn list_deleted_apartments(auth: AuthContext, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = apartments.filter(is_deleted.eq(true)).select(Apartment::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

// Restore apartment (Admin or Manager)
pub async fn restore_apartment(auth: AuthContext, path: web::Path<u64>, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as a_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    let id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(a_dsl::apartments.filter(a_dsl::id.eq(id))).set(a_dsl::is_deleted.eq(false)).execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/apartments", web::get().to(list_apartments))
        .route("/apartments", web::post().to(create_apartment))
        .route("/apartments/deleted", web::get().to(list_deleted_apartments))
        .route("/apartments/{id}/restore", web::post().to(restore_apartment))
        .route(
            "/buildings/{id}/apartments",
            web::get().to(list_building_apartments),
        )
        .route(
            "/apartments/{id}/owners",
            web::get().to(list_apartment_owners),
        )
        .route(
            "/apartments/{id}/owners",
            web::post().to(add_apartment_owner),
        )
        .route(
            "/apartments/{id}/owners/{user_id}",
            web::delete().to(remove_apartment_owner),
        )
        .route("/apartments/{id}", web::delete().to(delete_apartment));
}
