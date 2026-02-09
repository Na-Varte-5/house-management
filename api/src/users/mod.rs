use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{NewUser, PublicUser, User}; // added PublicUser import
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::schema::{roles as roles_schema, user_roles as ur_schema};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use serde::Serialize;
use utoipa;

/// Type alias for owned apartment query results
type OwnedApartmentRow = (
    u64,
    String,
    u64,
    String,
    Option<f64>,
    Option<i32>,
    Option<i32>,
);

/// Type alias for rented apartment query results
type RentedApartmentRow = (
    u64,
    String,
    u64,
    String,
    Option<f64>,
    Option<i32>,
    Option<i32>,
    Option<bool>,
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
);

/// List all users
///
/// Returns all users with complete information including password hashes.
/// Requires Admin role. For public user info, use /users/public endpoint.
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of users", body = PaginatedResponse<User>),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn list_users(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    use crate::schema::users::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let total = users.count().get_result::<i64>(&mut conn)?;
    let user_list = users
        .select(User::as_select())
        .limit(query.limit())
        .offset(query.offset())
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(PaginatedResponse::new(user_list, total, &query)))
}

/// Create a new user
///
/// Creates a new user. Requires Admin role. Password should already be hashed in the NewUser payload.
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = NewUser,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn create_user(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    item: web::Json<NewUser>,
) -> Result<impl Responder, AppError> {
    use crate::schema::users::dsl as users_dsl;
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::insert_into(users_dsl::users)
        .values(&*item)
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SetRolesRequest {
    pub roles: Vec<String>,
}
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UserRolesResponse {
    pub user_id: u64,
    pub roles: Vec<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct UserWithRoles {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

/// List all users with their roles
///
/// Returns all users along with their assigned roles. Requires Admin role.
#[utoipa::path(
    get,
    path = "/api/v1/users/with_roles",
    responses(
        (status = 200, description = "List of users with roles", body = Vec<UserWithRoles>),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn list_users_with_roles(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    use crate::schema::users::dsl as u;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let all: Vec<User> = u::users.select(User::as_select()).load(&mut conn)?;
    let mut out = Vec::with_capacity(all.len());
    for usr in all.into_iter() {
        let roles = crate::auth::roles::get_user_roles(usr.id, &mut conn);
        out.push(UserWithRoles {
            id: usr.id,
            email: usr.email,
            name: usr.name,
            roles,
        });
    }
    Ok(HttpResponse::Ok().json(out))
}

/// Set user roles
///
/// Sets the roles for a user, replacing all existing roles. Creates roles if they don't exist.
/// Valid roles: Admin, Manager, Homeowner, Renter, HOAMember. Requires Admin role.
#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/roles",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    request_body = SetRolesRequest,
    responses(
        (status = 200, description = "Roles set successfully", body = UserRolesResponse),
        (status = 400, description = "Bad request - invalid role name"),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn set_user_roles(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<SetRolesRequest>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    let allowed: std::collections::HashSet<&'static str> =
        ["Admin", "Manager", "Homeowner", "Renter", "HOAMember"]
            .into_iter()
            .collect();
    for r in &payload.roles {
        if !allowed.contains(r.as_str()) {
            return Err(AppError::BadRequest(format!("invalid_role:{}", r)));
        }
    }
    let user_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    for role_name in &payload.roles {
        let role_id_res: Result<u64, _> = roles_schema::dsl::roles
            .filter(roles_schema::dsl::name.eq(role_name))
            .select(roles_schema::dsl::id)
            .first(&mut conn);
        let role_id_val = match role_id_res {
            Ok(idv) => idv,
            Err(_) => {
                diesel::insert_into(roles_schema::dsl::roles)
                    .values(roles_schema::dsl::name.eq(role_name))
                    .execute(&mut conn)?;
                roles_schema::dsl::roles
                    .filter(roles_schema::dsl::name.eq(role_name))
                    .select(roles_schema::dsl::id)
                    .first(&mut conn)?
            }
        };
        let exists: Result<(u64, u64), _> = ur_schema::dsl::user_roles
            .filter(
                ur_schema::dsl::user_id
                    .eq(user_id)
                    .and(ur_schema::dsl::role_id.eq(role_id_val)),
            )
            .select((ur_schema::dsl::user_id, ur_schema::dsl::role_id))
            .first(&mut conn);
        if exists.is_err() {
            diesel::insert_into(ur_schema::dsl::user_roles)
                .values((
                    ur_schema::dsl::user_id.eq(user_id),
                    ur_schema::dsl::role_id.eq(role_id_val),
                ))
                .execute(&mut conn)?;
        }
    }
    // remove roles no longer present in payload
    use crate::schema::roles::dsl as rl;
    use crate::schema::user_roles::dsl as ur;
    let current: Vec<(u64, String)> = ur::user_roles
        .inner_join(rl::roles.on(rl::id.eq(ur::role_id)))
        .filter(ur::user_id.eq(user_id))
        .select((rl::id, rl::name))
        .load(&mut conn)?;
    let desired: std::collections::HashSet<&str> =
        payload.roles.iter().map(|s| s.as_str()).collect();
    for (rid, rname) in current {
        if !desired.contains(rname.as_str()) {
            diesel::delete(ur::user_roles.filter(ur::user_id.eq(user_id).and(ur::role_id.eq(rid))))
                .execute(&mut conn)?;
        }
    }
    let assigned: Vec<String> = ur_schema::dsl::user_roles
        .inner_join(roles_schema::dsl::roles.on(roles_schema::dsl::id.eq(ur_schema::dsl::role_id)))
        .filter(ur_schema::dsl::user_id.eq(user_id))
        .select(roles_schema::dsl::name)
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(UserRolesResponse {
        user_id,
        roles: assigned,
    }))
}

/// List users (public info only)
///
/// Returns all users with only public information (no password hashes).
/// Requires Admin, Manager role, or Homeowner role (for renter assignment).
#[utoipa::path(
    get,
    path = "/api/v1/users/public",
    responses(
        (status = 200, description = "List of users (public info)", body = Vec<PublicUser>),
        (status = 403, description = "Forbidden - requires Admin, Manager, or Homeowner role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn list_public_users(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager", "Homeowner"]) {
        return Err(AppError::Forbidden);
    }
    use crate::schema::users::dsl as u;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let raw: Vec<User> = u::users.select(User::as_select()).load(&mut conn)?;
    let list: Vec<PublicUser> = raw.into_iter().map(PublicUser::from).collect();
    Ok(HttpResponse::Ok().json(list))
}

/// Response structure for a single user property
#[derive(Serialize, utoipa::ToSchema)]
pub struct UserProperty {
    pub id: u64,
    pub apartment_number: String,
    pub building_id: u64,
    pub building_address: String,
    pub size_sq_m: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub relationship: String, // "owner" or "renter"
    pub is_active: bool,
    pub start_date: Option<String>, // for renters (YYYY-MM-DD format)
    pub end_date: Option<String>,   // for renters (YYYY-MM-DD format)
}

/// Statistics about user's properties
#[derive(Serialize, utoipa::ToSchema)]
pub struct PropertyStats {
    pub total_properties: usize,
    pub active_maintenance_requests: i64,
    pub pending_votes: i64,
}

/// Response for My Properties endpoint
#[derive(Serialize, utoipa::ToSchema)]
pub struct MyPropertiesResponse {
    pub properties: Vec<UserProperty>,
    pub stats: PropertyStats,
}

/// Get current user's properties
///
/// Returns all properties (apartments) owned or rented by the authenticated user,
/// along with statistics about maintenance requests and pending votes.
#[utoipa::path(
    get,
    path = "/api/v1/users/me/properties",
    responses(
        (status = 200, description = "User's properties and statistics", body = MyPropertiesResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users",
    security(("bearer_auth" = []))
)]
pub async fn get_my_properties(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::proposals::dsl as prop;
    use crate::schema::votes::dsl as v;

    let user_id = auth.user_id()?;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let mut properties: Vec<UserProperty> = Vec::new();

    // Get owned apartments
    let owned: Vec<OwnedApartmentRow> = apt::apartments
        .inner_join(ao::apartment_owners.on(ao::apartment_id.eq(apt::id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .filter(ao::user_id.eq(user_id))
        .filter(apt::is_deleted.eq(false))
        .filter(bld::is_deleted.eq(false))
        .select((
            apt::id,
            apt::number,
            apt::building_id,
            bld::address,
            apt::size_sq_m,
            apt::bedrooms,
            apt::bathrooms,
        ))
        .load(&mut conn)?;

    for (id, number, building_id, building_address, size_sq_m, bedrooms, bathrooms) in owned {
        properties.push(UserProperty {
            id,
            apartment_number: number,
            building_id,
            building_address,
            size_sq_m,
            bedrooms,
            bathrooms,
            relationship: "owner".to_string(),
            is_active: true,
            start_date: None,
            end_date: None,
        });
    }

    // Get rented apartments
    let rented: Vec<RentedApartmentRow> = apt::apartments
        .inner_join(ar::apartment_renters.on(ar::apartment_id.eq(apt::id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .filter(ar::user_id.eq(user_id))
        .filter(apt::is_deleted.eq(false))
        .filter(bld::is_deleted.eq(false))
        .select((
            apt::id,
            apt::number,
            apt::building_id,
            bld::address,
            apt::size_sq_m,
            apt::bedrooms,
            apt::bathrooms,
            ar::is_active,
            ar::start_date,
            ar::end_date,
        ))
        .load(&mut conn)?;

    for (
        id,
        number,
        building_id,
        building_address,
        size_sq_m,
        bedrooms,
        bathrooms,
        is_active,
        start_date,
        end_date,
    ) in rented
    {
        properties.push(UserProperty {
            id,
            apartment_number: number,
            building_id,
            building_address,
            size_sq_m,
            bedrooms,
            bathrooms,
            relationship: "renter".to_string(),
            is_active: is_active.unwrap_or(false),
            start_date: start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            end_date: end_date.map(|d| d.format("%Y-%m-%d").to_string()),
        });
    }

    // Get apartment IDs for statistics
    let apartment_ids: Vec<u64> = properties.iter().map(|p| p.id).collect();

    // Count active maintenance requests for user's apartments
    let active_maintenance_requests = if apartment_ids.is_empty() {
        0
    } else {
        mr::maintenance_requests
            .filter(mr::apartment_id.eq_any(&apartment_ids))
            .filter(
                mr::status
                    .eq("Open")
                    .or(mr::status.eq("InProgress"))
                    .or(mr::status.eq("Pending")),
            )
            .count()
            .get_result::<i64>(&mut conn)?
    };

    // Count pending votes for user's buildings
    // Get building IDs from user's properties
    let building_ids: Vec<u64> = properties.iter().map(|p| p.building_id).collect();

    let pending_votes = if building_ids.is_empty() {
        0
    } else {
        // Get open proposals for user's buildings
        let open_proposals: Vec<u64> = prop::proposals
            .filter(
                prop::building_id
                    .eq_any(&building_ids)
                    .or(prop::building_id.is_null()),
            ) // Include global proposals
            .filter(prop::status.eq("Open"))
            .select(prop::id)
            .load(&mut conn)?;

        if open_proposals.is_empty() {
            0
        } else {
            // Count proposals user hasn't voted on
            let voted_proposal_ids: Vec<u64> = v::votes
                .filter(v::user_id.eq(user_id))
                .filter(v::proposal_id.eq_any(&open_proposals))
                .select(v::proposal_id)
                .load(&mut conn)?;

            (open_proposals.len() - voted_proposal_ids.len()) as i64
        }
    };

    let total_properties = properties.len();

    Ok(HttpResponse::Ok().json(MyPropertiesResponse {
        properties,
        stats: PropertyStats {
            total_properties,
            active_maintenance_requests,
            pending_votes,
        },
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/users", web::get().to(list_users))
        .route("/users", web::post().to(create_user))
        .route("/users/public", web::get().to(list_public_users))
        .route("/users/with_roles", web::get().to(list_users_with_roles))
        .route("/users/me/properties", web::get().to(get_my_properties))
        .route("/users/{id}/roles", web::post().to(set_user_roles));
}
