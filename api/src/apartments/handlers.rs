use super::helpers::{ensure_user_has_role, log_property_event, remove_role_if_no_assignments};
use super::types::{
    ApartmentDetail, ApartmentPermissions, ApartmentWithBuilding, OwnerAssignPayload,
    PropertyHistoryRow,
};
use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{
    Apartment, ApartmentOwner, NewApartment, PropertyHistoryEnriched, PublicUser, User,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

/// List all active apartments
///
/// Returns a paginated list of all apartments across all buildings that have not been soft-deleted.
/// Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/apartments",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of apartments", body = PaginatedResponse<Apartment>),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_apartments(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    use crate::schema::apartments::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let total = apartments
        .filter(is_deleted.eq(false))
        .count()
        .get_result::<i64>(&mut conn)?;
    let list = apartments
        .filter(is_deleted.eq(false))
        .select(Apartment::as_select())
        .limit(query.limit())
        .offset(query.offset())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(list, total, &query)))
}

/// List apartments for a specific building
///
/// Returns all apartments in a specific building that haven't been soft-deleted.
/// Requires authentication. Users can only see apartments in buildings they have access to.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}/apartments",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of apartments in the building", body = Vec<Apartment>),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - no access to this building"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_building_apartments(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl::*;
    let building = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Check if user has access to this building
    let is_admin = auth.has_any_role(&["Admin", "Manager"]);
    let user_id = auth.user_id()?;

    use crate::auth::building_access::get_user_building_ids;
    let maybe_building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    // If Some(vec), user can only see those buildings; if None, user is admin and can see all
    if let Some(accessible_buildings) = maybe_building_ids
        && !accessible_buildings.contains(&building)
    {
        return Err(AppError::Forbidden);
    }

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

/// List owners of an apartment
///
/// Returns all users who are registered as owners of the specified apartment.
/// Requires Admin or Manager role for privacy protection.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/owners",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of apartment owners", body = Vec<PublicUser>),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_apartment_owners(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
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
    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
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
        // Already exists, but ensure role is assigned
        ensure_user_has_role(new.user_id, "Homeowner", &mut conn).await?;
        return Ok(HttpResponse::NoContent().finish());
    }
    diesel::insert_into(ao::apartment_owners)
        .values(&new)
        .execute(&mut conn)?;

    // Auto-assign Homeowner role
    ensure_user_has_role(new.user_id, "Homeowner", &mut conn).await?;

    // Log property history event
    use crate::schema::users::dsl as users;
    let user_name: String = users::users
        .filter(users::id.eq(new.user_id))
        .select(users::name)
        .first(&mut conn)?;

    log_property_event(
        apartment,
        "owner_added",
        Some(new.user_id),
        current_user_id,
        format!("Added {} as owner", user_name),
        None,
        &mut conn,
    )
    .await?;

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
    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let (apartment, user) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get user name before deletion for logging
    use crate::schema::users::dsl as users;
    let user_name: String = users::users
        .filter(users::id.eq(user))
        .select(users::name)
        .first(&mut conn)?;

    diesel::delete(
        ao::apartment_owners.filter(ao::apartment_id.eq(apartment).and(ao::user_id.eq(user))),
    )
    .execute(&mut conn)?;

    // Auto-remove Homeowner role if user has no more ownership assignments
    remove_role_if_no_assignments(user, "Homeowner", &mut conn).await?;

    // Log property history event
    log_property_event(
        apartment,
        "owner_removed",
        Some(user),
        current_user_id,
        format!("Removed {} as owner", user_name),
        None,
        &mut conn,
    )
    .await?;

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

/// Get apartment details
///
/// Returns detailed information about a specific apartment including building info.
/// Requires authentication and either Admin/Manager role or ownership/renter status.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "Apartment details", body = ApartmentDetail),
        (status = 401, description = "Unauthorized - requires authentication"),
        (status = 403, description = "Forbidden - no access to this apartment"),
        (status = 404, description = "Apartment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn get_apartment(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::apartments::dsl as a;
    use crate::schema::buildings::dsl as b;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    // Check if apartment exists and is not deleted
    let apartment_data: Result<(u64, String, u64, Option<f64>), _> = a::apartments
        .filter(a::id.eq(apartment_id).and(a::is_deleted.eq(false)))
        .select((a::id, a::number, a::building_id, a::size_sq_m))
        .first(&mut conn);

    let (id, number, building_id, size_sq_m) = match apartment_data {
        Ok(data) => data,
        Err(_) => return Err(AppError::NotFound),
    };

    // Get building address
    let building_address: String = b::buildings
        .filter(b::id.eq(building_id))
        .select(b::address)
        .first(&mut conn)
        .map_err(|_| AppError::Internal("building_not_found".into()))?;

    // Check access: Admin/Manager always has access, or user must be owner/renter
    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let is_owner: bool = ao::apartment_owners
        .filter(
            ao::apartment_id
                .eq(apartment_id)
                .and(ao::user_id.eq(user_id)),
        )
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;

    let is_renter: bool = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(user_id))
                .and(ar::is_active.eq(true)),
        )
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;

    if !is_admin_or_manager && !is_owner && !is_renter {
        return Err(AppError::Forbidden);
    }

    Ok(HttpResponse::Ok().json(ApartmentDetail {
        id,
        number,
        building_id,
        building_address,
        size_sq_m,
    }))
}

/// Get apartment permissions for current user
///
/// Returns what actions the current user can perform on a specific apartment.
/// Requires authentication.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/permissions",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "User permissions for the apartment", body = ApartmentPermissions),
        (status = 401, description = "Unauthorized - requires authentication"),
        (status = 404, description = "Apartment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn get_apartment_permissions(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::apartments::dsl as a;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    // Check if apartment exists
    let exists: i64 = a::apartments
        .filter(a::id.eq(apartment_id).and(a::is_deleted.eq(false)))
        .count()
        .get_result(&mut conn)?;

    if exists == 0 {
        return Err(AppError::NotFound);
    }

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let is_owner: bool = ao::apartment_owners
        .filter(
            ao::apartment_id
                .eq(apartment_id)
                .and(ao::user_id.eq(user_id)),
        )
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;

    let is_renter: bool = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(user_id))
                .and(ar::is_active.eq(true)),
        )
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;

    let can_view = is_admin_or_manager || is_owner || is_renter;
    let can_manage_renters = is_admin_or_manager || is_owner;
    let can_view_meters = is_admin_or_manager || is_owner || is_renter;

    Ok(HttpResponse::Ok().json(ApartmentPermissions {
        can_view,
        can_manage_renters,
        can_view_meters,
        is_owner,
        is_renter,
    }))
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

/// Get property history for an apartment
///
/// Returns chronological history of owner and renter changes for an apartment.
/// Requires authentication.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/history",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "Property history", body = Vec<PropertyHistoryEnriched>),
        (status = 404, description = "Apartment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn get_apartment_history(
    _auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::property_history::dsl as ph;
    use crate::schema::users::dsl as users;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Fetch history events with manual field selection
    let history_data: Vec<PropertyHistoryRow> = ph::property_history
        .filter(ph::apartment_id.eq(apartment_id))
        .select((
            ph::id,
            ph::apartment_id,
            ph::event_type,
            ph::user_id,
            ph::changed_by,
            ph::description,
            ph::metadata,
            ph::created_at,
        ))
        .order(ph::created_at.desc())
        .load(&mut conn)?;

    // Enrich with user names
    let mut enriched: Vec<PropertyHistoryEnriched> = Vec::new();
    for (id, apt_id, event_type, user_id, changed_by, description, metadata, created_at) in
        history_data
    {
        // Get changed_by user name
        let changed_by_data: (u64, String, String) = users::users
            .filter(users::id.eq(changed_by))
            .select((users::id, users::email, users::name))
            .first(&mut conn)?;
        let changed_by_name = changed_by_data.2;

        // Get affected user name if applicable
        let user_name = if let Some(uid) = user_id {
            let user_data: Result<(u64, String, String), _> = users::users
                .filter(users::id.eq(uid))
                .select((users::id, users::email, users::name))
                .first(&mut conn);
            user_data.ok().map(|u| u.2)
        } else {
            None
        };

        enriched.push(PropertyHistoryEnriched {
            id,
            apartment_id: apt_id,
            event_type,
            user_id,
            user_name,
            changed_by,
            changed_by_name,
            description,
            metadata,
            created_at,
        });
    }

    Ok(HttpResponse::Ok().json(enriched))
}
