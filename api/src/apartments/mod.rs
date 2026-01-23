use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{Apartment, ApartmentOwner, NewApartment, PublicUser, User};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use utoipa;

/// List all active apartments
///
/// Returns a list of all apartments across all buildings that have not been soft-deleted.
#[utoipa::path(
    get,
    path = "/api/v1/apartments",
    responses(
        (status = 200, description = "List of apartments", body = Vec<Apartment>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments"
)]
pub async fn list_apartments(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = apartments
        .filter(is_deleted.eq(false))
        .select(Apartment::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

/// List apartments for a specific building
///
/// Returns all apartments in a specific building that haven't been soft-deleted.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}/apartments",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of apartments in the building", body = Vec<Apartment>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments"
)]
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

/// List apartments for a specific building (user-filtered)
///
/// For Admin/Manager roles, returns all active apartments in the building.
/// For other users, returns only apartments they own in the building.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}/apartments/my",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of user's apartments in the building", body = Vec<Apartment>),
        (status = 401, description = "Unauthorized - requires authentication"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_my_building_apartments(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao_dsl;
    use crate::schema::apartments::dsl::*;

    let building = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Admin/Manager can see all apartments in the building
    if auth.has_any_role(&["Admin", "Manager"]) {
        let list = apartments
            .filter(building_id.eq(building).and(is_deleted.eq(false)))
            .select(Apartment::as_select())
            .load(&mut conn)?;
        return Ok(HttpResponse::Ok().json(list));
    }

    // For regular users, get only their apartments in this building
    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let list = apartments
        .inner_join(ao_dsl::apartment_owners.on(ao_dsl::apartment_id.eq(id)))
        .filter(
            building_id
                .eq(building)
                .and(is_deleted.eq(false))
                .and(ao_dsl::user_id.eq(user_id)),
        )
        .select(Apartment::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(list))
}

/// Create a new apartment
///
/// Creates a new apartment in a building. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/apartments",
    request_body = NewApartment,
    responses(
        (status = 201, description = "Apartment created successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
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

    // Get the inserted apartment
    let inserted_id: u64 = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
    >("LAST_INSERT_ID()"))
    .first(&mut conn)?;

    let apartment: Apartment = a_dsl::apartments
        .filter(a_dsl::id.eq(inserted_id))
        .select(Apartment::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Created().json(apartment))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct OwnerAssignPayload {
    pub user_id: u64,
}

/// List owners of an apartment
///
/// Returns all users who are registered as owners of the specified apartment.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/owners",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of apartment owners", body = Vec<PublicUser>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments"
)]
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

/// Assign an owner to an apartment
///
/// Adds a user as an owner of the specified apartment. This operation is idempotent -
/// if the user is already an owner, returns 204 without error. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/apartments/{id}/owners",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    request_body = OwnerAssignPayload,
    responses(
        (status = 201, description = "Owner assigned successfully"),
        (status = 204, description = "Owner already assigned (idempotent)"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
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

/// Remove an owner from an apartment
///
/// Removes a user's ownership assignment from the specified apartment.
/// Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/apartments/{id}/owners/{user_id}",
    params(
        ("id" = u64, Path, description = "Apartment ID"),
        ("user_id" = u64, Path, description = "User ID to remove as owner")
    ),
    responses(
        (status = 204, description = "Owner removed successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
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

/// Soft-delete an apartment
///
/// Marks an apartment as deleted (soft-delete). Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/apartments/{id}",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 204, description = "Apartment deleted successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Apartment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn delete_apartment(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as a_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(a_dsl::apartments.filter(a_dsl::id.eq(id)))
        .set(a_dsl::is_deleted.eq(true))
        .execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

/// List soft-deleted apartments
///
/// Returns a list of apartments that have been soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/deleted",
    responses(
        (status = 200, description = "List of deleted apartments", body = Vec<Apartment>),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_deleted_apartments(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = apartments
        .filter(is_deleted.eq(true))
        .select(Apartment::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

/// Restore a soft-deleted apartment
///
/// Restores an apartment that was previously soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/apartments/{id}/restore",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "Apartment restored successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Apartment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn restore_apartment(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as a_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(a_dsl::apartments.filter(a_dsl::id.eq(id)))
        .set(a_dsl::is_deleted.eq(false))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

/// Apartment with building info for forms
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ApartmentWithBuilding {
    pub id: u64,
    pub number: String,
    pub building_id: u64,
    pub building_address: String,
}

/// List user's apartments with building info
///
/// Returns apartments owned by the current user, enriched with building information.
/// Used for forms where users need to select their apartments.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/my",
    responses(
        (status = 200, description = "List of user's apartments with building info", body = Vec<ApartmentWithBuilding>),
        (status = 401, description = "Unauthorized - requires authentication"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_my_apartments(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao_dsl;
    use crate::schema::apartments::dsl as a_dsl;
    use crate::schema::buildings::dsl as b_dsl;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let apartments_with_ids: Vec<(u64, String, u64, String)> = a_dsl::apartments
        .inner_join(ao_dsl::apartment_owners.on(ao_dsl::apartment_id.eq(a_dsl::id)))
        .inner_join(b_dsl::buildings.on(b_dsl::id.eq(a_dsl::building_id)))
        .filter(
            a_dsl::is_deleted
                .eq(false)
                .and(b_dsl::is_deleted.eq(false))
                .and(ao_dsl::user_id.eq(user_id)),
        )
        .select((a_dsl::id, a_dsl::number, a_dsl::building_id, b_dsl::address))
        .load(&mut conn)?;

    let enriched: Vec<ApartmentWithBuilding> = apartments_with_ids
        .into_iter()
        .map(
            |(id, number, building_id, building_address)| ApartmentWithBuilding {
                id,
                number,
                building_id,
                building_address,
            },
        )
        .collect();

    Ok(HttpResponse::Ok().json(enriched))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/apartments", web::get().to(list_apartments))
        .route("/apartments", web::post().to(create_apartment))
        .route("/apartments/my", web::get().to(list_my_apartments))
        .route(
            "/apartments/deleted",
            web::get().to(list_deleted_apartments),
        )
        .route(
            "/apartments/{id}/restore",
            web::post().to(restore_apartment),
        )
        .route(
            "/buildings/{id}/apartments",
            web::get().to(list_building_apartments),
        )
        .route(
            "/buildings/{id}/apartments/my",
            web::get().to(list_my_building_apartments),
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
