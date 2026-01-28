use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{Building, NewBuilding, User};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use utoipa;

/// List all active buildings
///
/// Returns a list of all buildings that have not been soft-deleted.
/// Requires authentication. Users see buildings they have access to.
#[utoipa::path(
    get,
    path = "/api/v1/buildings",
    responses(
        (status = 200, description = "List of buildings", body = Vec<Building>),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn list_buildings(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Admin/Manager see all buildings, others see only their buildings
    let is_admin = auth.has_any_role(&["Admin", "Manager"]);
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    use crate::auth::building_access::get_user_building_ids;
    let maybe_building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    let list = match maybe_building_ids {
        None => {
            // Admin/Manager - see all buildings
            buildings
                .filter(is_deleted.eq(false))
                .select(Building::as_select())
                .load(&mut conn)?
        }
        Some(building_ids) => {
            // Regular user - see only accessible buildings
            buildings
                .filter(id.eq_any(building_ids).and(is_deleted.eq(false)))
                .select(Building::as_select())
                .load(&mut conn)?
        }
    };

    Ok(HttpResponse::Ok().json(list))
}

/// Get a single building by ID
///
/// Returns a single building by its ID if it exists and hasn't been soft-deleted.
/// Requires authentication. Users can only view buildings they have access to.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "Building details", body = Building),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - no access to this building"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn get_building(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl::*;
    let building_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Check if user has access to this building
    let is_admin = auth.has_any_role(&["Admin", "Manager"]);
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    use crate::auth::building_access::get_user_building_ids;
    let maybe_building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    // If Some(vec), user can only see those buildings; if None, user is admin and can see all
    if let Some(accessible_buildings) = maybe_building_ids
        && !accessible_buildings.contains(&building_id)
    {
        return Err(AppError::Forbidden);
    }

    let building = buildings
        .filter(id.eq(building_id).and(is_deleted.eq(false)))
        .select(Building::as_select())
        .first(&mut conn)
        .optional()?;

    match building {
        Some(b) => Ok(HttpResponse::Ok().json(b)),
        None => Err(AppError::NotFound),
    }
}

/// Create a new building
///
/// Creates a new building. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/buildings",
    request_body = NewBuilding,
    responses(
        (status = 201, description = "Building created successfully", body = Building),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
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

    // Get the inserted building
    let inserted_id: u64 = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
    >("LAST_INSERT_ID()"))
    .first(&mut conn)?;

    let building: Building = b_dsl::buildings
        .filter(b_dsl::id.eq(inserted_id))
        .select(Building::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Created().json(building))
}

/// Soft-delete a building
///
/// Marks a building as deleted (soft-delete). Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/buildings/{id}",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 204, description = "Building deleted successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn delete_building(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl as b_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(b_dsl::buildings.filter(b_dsl::id.eq(id)))
        .set(b_dsl::is_deleted.eq(true))
        .execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

/// List soft-deleted buildings
///
/// Returns a list of buildings that have been soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/deleted",
    responses(
        (status = 200, description = "List of deleted buildings", body = Vec<Building>),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn list_deleted_buildings(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl::*;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let list = buildings
        .filter(is_deleted.eq(true))
        .select(Building::as_select())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(list))
}

/// Restore a soft-deleted building
///
/// Restores a building that was previously soft-deleted. Requires Admin or Manager role.
#[utoipa::path(
    post,
    path = "/api/v1/buildings/{id}/restore",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "Building restored successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn restore_building(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::buildings::dsl as b_dsl;
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::update(b_dsl::buildings.filter(b_dsl::id.eq(id)))
        .set(b_dsl::is_deleted.eq(false))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

/// List buildings associated with the current user
///
/// For Admin/Manager roles, returns all active buildings.
/// For other users, returns only buildings where they own or rent an apartment.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/my",
    responses(
        (status = 200, description = "List of user's associated buildings", body = Vec<Building>),
        (status = 401, description = "Unauthorized - requires authentication"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn list_my_buildings(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::auth::get_user_building_ids;
    use crate::schema::buildings::dsl::*;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let is_admin = auth.has_any_role(&["Admin"]);

    // Get user's accessible buildings (includes owners, renters, and managers)
    let building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    let list = if let Some(ref ids) = building_ids {
        // User has restricted access - filter by accessible buildings
        buildings
            .filter(is_deleted.eq(false).and(id.eq_any(ids)))
            .select(Building::as_select())
            .load(&mut conn)?
    } else {
        // Admin - see all buildings
        buildings
            .filter(is_deleted.eq(false))
            .select(Building::as_select())
            .load(&mut conn)?
    };

    Ok(HttpResponse::Ok().json(list))
}

/// List managers for a building (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{id}/managers",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of managers", body = Vec<User>),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn list_building_managers(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::models::User;
    use crate::schema::{building_managers, users};

    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let building_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let managers: Vec<User> = building_managers::table
        .inner_join(users::table.on(users::id.eq(building_managers::user_id)))
        .filter(building_managers::building_id.eq(building_id))
        .select(User::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(managers))
}

/// Assign a manager to a building (Admin only)
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct AssignManagerPayload {
    pub user_id: u64,
}

#[utoipa::path(
    post,
    path = "/api/v1/buildings/{id}/managers",
    params(
        ("id" = u64, Path, description = "Building ID")
    ),
    request_body = AssignManagerPayload,
    responses(
        (status = 200, description = "Manager assigned successfully"),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 404, description = "Building or user not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn assign_building_manager(
    auth: AuthContext,
    path: web::Path<u64>,
    payload: web::Json<AssignManagerPayload>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::models::BuildingManager;
    use crate::schema::{building_managers, buildings, users};

    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let building_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Verify building exists
    buildings::table
        .find(building_id)
        .first::<Building>(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    // Verify user exists
    users::table
        .find(payload.user_id)
        .first::<User>(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    // Insert manager assignment (ignore if already exists)
    let new_manager = BuildingManager {
        building_id,
        user_id: payload.user_id,
        created_at: None,
    };

    diesel::insert_into(building_managers::table)
        .values(&new_manager)
        .execute(&mut conn)
        .or_else(|e| {
            // Ignore duplicate key errors
            if e.to_string().contains("Duplicate entry") {
                Ok(0)
            } else {
                Err(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Manager assigned successfully"})))
}

/// Remove a manager from a building (Admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/buildings/{id}/managers/{user_id}",
    params(
        ("id" = u64, Path, description = "Building ID"),
        ("user_id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Manager removed successfully"),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Buildings",
    security(("bearer_auth" = []))
)]
pub async fn remove_building_manager(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::building_managers;

    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let (building_id, user_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    diesel::delete(
        building_managers::table
            .filter(building_managers::building_id.eq(building_id))
            .filter(building_managers::user_id.eq(user_id)),
    )
    .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Manager removed successfully"})))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/buildings", web::get().to(list_buildings))
        .route("/buildings", web::post().to(create_building))
        .route("/buildings/my", web::get().to(list_my_buildings))
        .route("/buildings/deleted", web::get().to(list_deleted_buildings))
        .route("/buildings/{id}", web::get().to(get_building))
        .route("/buildings/{id}/restore", web::post().to(restore_building))
        .route("/buildings/{id}", web::delete().to(delete_building))
        .route(
            "/buildings/{id}/managers",
            web::get().to(list_building_managers),
        )
        .route(
            "/buildings/{id}/managers",
            web::post().to(assign_building_manager),
        )
        .route(
            "/buildings/{id}/managers/{user_id}",
            web::delete().to(remove_building_manager),
        );
}
