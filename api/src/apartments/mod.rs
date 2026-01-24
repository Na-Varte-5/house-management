use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{
    Apartment, ApartmentOwner, ApartmentRenter, NewApartment, NewApartmentRenter,
    NewPropertyHistory, PropertyHistoryEnriched, PublicUser, User,
};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use utoipa;

/// Helper function to ensure a user has a specific role.
/// Creates the role if it doesn't exist, then assigns it to the user if not already assigned.
async fn ensure_user_has_role(
    user_id: u64,
    role_name: &str,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::user_roles::dsl as ur_schema;

    // Get or create role
    let role_id_res: Result<u64, _> = roles_schema::roles
        .filter(roles_schema::name.eq(role_name))
        .select(roles_schema::id)
        .first(conn);

    let role_id = match role_id_res {
        Ok(id) => id,
        Err(_) => {
            // Create the role if it doesn't exist
            diesel::insert_into(roles_schema::roles)
                .values(roles_schema::name.eq(role_name))
                .execute(conn)?;
            roles_schema::roles
                .filter(roles_schema::name.eq(role_name))
                .select(roles_schema::id)
                .first(conn)?
        }
    };

    // Check if user already has this role
    let exists: Result<(u64, u64), _> = ur_schema::user_roles
        .filter(
            ur_schema::user_id
                .eq(user_id)
                .and(ur_schema::role_id.eq(role_id)),
        )
        .select((ur_schema::user_id, ur_schema::role_id))
        .first(conn);

    // Assign role if not already assigned
    if exists.is_err() {
        diesel::insert_into(ur_schema::user_roles)
            .values((
                ur_schema::user_id.eq(user_id),
                ur_schema::role_id.eq(role_id),
            ))
            .execute(conn)?;
    }

    Ok(())
}

/// Helper function to remove a role from a user if they have no more property assignments.
/// Checks if the user owns any apartments or is an active renter.
/// If no assignments exist, removes the specified role.
async fn remove_role_if_no_assignments(
    user_id: u64,
    role_name: &str,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::user_roles::dsl as ur_schema;

    // Determine which tables to check based on role
    let has_assignments = match role_name {
        "Homeowner" => {
            // Check if user owns any apartments
            let count: i64 = ao::apartment_owners
                .filter(ao::user_id.eq(user_id))
                .count()
                .get_result(conn)?;
            count > 0
        }
        "Renter" => {
            // Check if user has any active rental assignments
            let count: i64 = ar::apartment_renters
                .filter(ar::user_id.eq(user_id).and(ar::is_active.eq(true)))
                .count()
                .get_result(conn)?;
            count > 0
        }
        _ => return Ok(()), // Don't auto-remove other roles
    };

    // If no assignments, remove the role
    if !has_assignments {
        let role_id_res: Result<u64, _> = roles_schema::roles
            .filter(roles_schema::name.eq(role_name))
            .select(roles_schema::id)
            .first(conn);

        if let Ok(role_id) = role_id_res {
            diesel::delete(
                ur_schema::user_roles.filter(
                    ur_schema::user_id
                        .eq(user_id)
                        .and(ur_schema::role_id.eq(role_id)),
                ),
            )
            .execute(conn)?;
        }
    }

    Ok(())
}

/// Helper function to log property history events
async fn log_property_event(
    apartment_id: u64,
    event_type: &str,
    user_id: Option<u64>,
    changed_by: u64,
    description: String,
    metadata: Option<String>,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::property_history::dsl as ph;

    let new_event = NewPropertyHistory {
        apartment_id,
        event_type: event_type.to_string(),
        user_id,
        changed_by,
        description,
        metadata,
    };

    diesel::insert_into(ph::property_history)
        .values(&new_event)
        .execute(conn)?;

    Ok(())
}

/// List all active apartments
///
/// Returns a list of all apartments across all buildings that have not been soft-deleted.
/// Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/apartments",
    responses(
        (status = 200, description = "List of apartments", body = Vec<Apartment>),
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
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
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
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    use crate::auth::building_access::get_user_building_ids;
    let maybe_building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    // If Some(vec), user can only see those buildings
    if let Some(accessible_buildings) = maybe_building_ids {
        if !accessible_buildings.contains(&building) {
            return Err(AppError::Forbidden);
        }
    }
    // If None, user is admin and can see all buildings

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

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RenterAssignPayload {
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RenterUpdatePayload {
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RenterWithUser {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub user: PublicUser,
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

/// List renters of an apartment
///
/// Returns all users who are registered as renters of the specified apartment.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/renters",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of renters", body = Vec<RenterWithUser>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_apartment_renters(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::users::dsl as users;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Fetch apartment renters with manual field selection
    let renters_data: Vec<(
        u64,
        u64,
        u64,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        Option<bool>,
        Option<chrono::NaiveDateTime>,
    )> = ar::apartment_renters
        .filter(ar::apartment_id.eq(apartment_id))
        .select((
            ar::id,
            ar::apartment_id,
            ar::user_id,
            ar::start_date,
            ar::end_date,
            ar::is_active,
            ar::created_at,
        ))
        .load(&mut conn)?;

    // Fetch user details for each renter
    let mut result: Vec<RenterWithUser> = Vec::new();
    for (id, apt_id, user_id, start_date, end_date, is_active, created_at) in renters_data {
        let user_data: (u64, String, String) = users::users
            .filter(users::id.eq(user_id))
            .select((users::id, users::email, users::name))
            .first(&mut conn)?;

        result.push(RenterWithUser {
            id,
            apartment_id: apt_id,
            user_id,
            start_date,
            end_date,
            is_active: is_active.unwrap_or(false),
            created_at,
            user: PublicUser {
                id: user_data.0,
                email: user_data.1,
                name: user_data.2,
            },
        });
    }

    Ok(HttpResponse::Ok().json(result))
}

/// Add a renter to an apartment
///
/// Assigns a user as a renter of the specified apartment with rental period dates.
/// Automatically assigns the Renter role to the user.
/// Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/apartments/{id}/renters",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    request_body = RenterAssignPayload,
    responses(
        (status = 201, description = "Renter assigned successfully", body = ApartmentRenter),
        (status = 204, description = "Renter already assigned"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn add_apartment_renter(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<RenterAssignPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_renters::dsl as ar;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Check if renter assignment already exists
    let exists: Result<(u64, u64, u64), _> = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(payload.user_id)),
        )
        .select((ar::id, ar::apartment_id, ar::user_id))
        .first(&mut conn);

    if exists.is_ok() {
        // Already exists, but ensure role is assigned if active
        if payload.is_active.unwrap_or(true) {
            ensure_user_has_role(payload.user_id, "Renter", &mut conn).await?;
        }
        return Ok(HttpResponse::NoContent().finish());
    }

    let new = NewApartmentRenter {
        apartment_id,
        user_id: payload.user_id,
        start_date: payload.start_date,
        end_date: payload.end_date,
        is_active: payload.is_active,
    };

    diesel::insert_into(ar::apartment_renters)
        .values(&new)
        .execute(&mut conn)?;

    // Fetch created renter with manual field selection
    let renter_data: (
        u64,
        u64,
        u64,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        Option<bool>,
        Option<chrono::NaiveDateTime>,
    ) = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(payload.user_id)),
        )
        .select((
            ar::id,
            ar::apartment_id,
            ar::user_id,
            ar::start_date,
            ar::end_date,
            ar::is_active,
            ar::created_at,
        ))
        .first(&mut conn)?;

    let is_active = renter_data.5.unwrap_or(false);

    // Auto-assign Renter role if active
    if is_active {
        ensure_user_has_role(payload.user_id, "Renter", &mut conn).await?;
    }

    // Log property history event
    use crate::schema::users::dsl as users;
    let user_name: String = users::users
        .filter(users::id.eq(payload.user_id))
        .select(users::name)
        .first(&mut conn)?;

    let mut metadata_parts = Vec::new();
    if let Some(start) = renter_data.3 {
        metadata_parts.push(format!("start_date: {}", start));
    }
    if let Some(end) = renter_data.4 {
        metadata_parts.push(format!("end_date: {}", end));
    }
    metadata_parts.push(format!("active: {}", is_active));

    let metadata = Some(
        serde_json::json!({
            "start_date": renter_data.3.map(|d| d.to_string()),
            "end_date": renter_data.4.map(|d| d.to_string()),
            "is_active": is_active,
        })
        .to_string(),
    );

    log_property_event(
        apartment_id,
        "renter_added",
        Some(payload.user_id),
        current_user_id,
        format!(
            "Added {} as renter ({})",
            user_name,
            metadata_parts.join(", ")
        ),
        metadata,
        &mut conn,
    )
    .await?;

    let renter = ApartmentRenter {
        id: renter_data.0,
        apartment_id: renter_data.1,
        user_id: renter_data.2,
        start_date: renter_data.3,
        end_date: renter_data.4,
        is_active,
        created_at: renter_data.6,
    };

    Ok(HttpResponse::Created().json(renter))
}

/// Update a renter assignment
///
/// Updates the rental period dates or active status for a renter assignment.
/// Manages Renter role based on active status.
/// Requires Admin or Manager role.
#[utoipa::path(
    put,
    path = "/api/v1/apartments/{id}/renters/{user_id}",
    params(
        ("id" = u64, Path, description = "Apartment ID"),
        ("user_id" = u64, Path, description = "User ID")
    ),
    request_body = RenterUpdatePayload,
    responses(
        (status = 200, description = "Renter updated successfully", body = ApartmentRenter),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Renter not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn update_apartment_renter(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
    payload: web::Json<RenterUpdatePayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_renters::dsl as ar;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let (apartment_id, user_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Build update query dynamically based on provided fields
    let target = ar::apartment_renters.filter(
        ar::apartment_id
            .eq(apartment_id)
            .and(ar::user_id.eq(user_id)),
    );

    if let Some(start) = payload.start_date {
        diesel::update(target.clone())
            .set(ar::start_date.eq(Some(start)))
            .execute(&mut conn)?;
    }

    if let Some(end) = payload.end_date {
        diesel::update(target.clone())
            .set(ar::end_date.eq(Some(end)))
            .execute(&mut conn)?;
    }

    if let Some(active) = payload.is_active {
        diesel::update(target.clone())
            .set(ar::is_active.eq(Some(active)))
            .execute(&mut conn)?;

        // Manage role based on active status
        if active {
            ensure_user_has_role(user_id, "Renter", &mut conn).await?;
        } else {
            remove_role_if_no_assignments(user_id, "Renter", &mut conn).await?;
        }
    }

    // Fetch and return updated renter with manual field selection
    let renter_data: (
        u64,
        u64,
        u64,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        Option<bool>,
        Option<chrono::NaiveDateTime>,
    ) = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(user_id)),
        )
        .select((
            ar::id,
            ar::apartment_id,
            ar::user_id,
            ar::start_date,
            ar::end_date,
            ar::is_active,
            ar::created_at,
        ))
        .first(&mut conn)?;

    // Log property history event
    use crate::schema::users::dsl as users;
    let user_name: String = users::users
        .filter(users::id.eq(user_id))
        .select(users::name)
        .first(&mut conn)?;

    let mut changes = Vec::new();
    if payload.start_date.is_some() {
        changes.push(format!("start_date: {:?}", renter_data.3));
    }
    if payload.end_date.is_some() {
        changes.push(format!("end_date: {:?}", renter_data.4));
    }
    if let Some(active) = payload.is_active {
        changes.push(format!("active: {}", active));
    }

    if !changes.is_empty() {
        let metadata = Some(
            serde_json::json!({
                "start_date": renter_data.3.map(|d| d.to_string()),
                "end_date": renter_data.4.map(|d| d.to_string()),
                "is_active": renter_data.5,
            })
            .to_string(),
        );

        log_property_event(
            apartment_id,
            "renter_updated",
            Some(user_id),
            current_user_id,
            format!("Updated renter {} ({})", user_name, changes.join(", ")),
            metadata,
            &mut conn,
        )
        .await?;
    }

    let renter = ApartmentRenter {
        id: renter_data.0,
        apartment_id: renter_data.1,
        user_id: renter_data.2,
        start_date: renter_data.3,
        end_date: renter_data.4,
        is_active: renter_data.5.unwrap_or(false),
        created_at: renter_data.6,
    };

    Ok(HttpResponse::Ok().json(renter))
}

/// Remove a renter from an apartment
///
/// Removes a user's renter assignment from the specified apartment.
/// Automatically removes Renter role if no other active rental assignments exist.
/// Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/apartments/{id}/renters/{user_id}",
    params(
        ("id" = u64, Path, description = "Apartment ID"),
        ("user_id" = u64, Path, description = "User ID to remove as renter")
    ),
    responses(
        (status = 204, description = "Renter removed successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn remove_apartment_renter(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_renters::dsl as ar;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let (apartment_id, user_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get user name before deletion for logging
    use crate::schema::users::dsl as users;
    let user_name: String = users::users
        .filter(users::id.eq(user_id))
        .select(users::name)
        .first(&mut conn)?;

    diesel::delete(
        ar::apartment_renters.filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(user_id)),
        ),
    )
    .execute(&mut conn)?;

    // Auto-remove Renter role if user has no more active rental assignments
    remove_role_if_no_assignments(user_id, "Renter", &mut conn).await?;

    // Log property history event
    log_property_event(
        apartment_id,
        "renter_removed",
        Some(user_id),
        current_user_id,
        format!("Removed {} as renter", user_name),
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
    let history_data: Vec<(
        u64,
        u64,
        String,
        Option<u64>,
        u64,
        String,
        Option<String>,
        Option<chrono::NaiveDateTime>,
    )> = ph::property_history
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
        .route(
            "/apartments/{id}/renters",
            web::get().to(list_apartment_renters),
        )
        .route(
            "/apartments/{id}/renters",
            web::post().to(add_apartment_renter),
        )
        .route(
            "/apartments/{id}/renters/{user_id}",
            web::put().to(update_apartment_renter),
        )
        .route(
            "/apartments/{id}/renters/{user_id}",
            web::delete().to(remove_apartment_renter),
        )
        .route(
            "/apartments/{id}/history",
            web::get().to(get_apartment_history),
        )
        .route("/apartments/{id}", web::delete().to(delete_apartment));
}
