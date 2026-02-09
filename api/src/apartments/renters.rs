use super::helpers::{ensure_user_has_role, log_property_event, remove_role_if_no_assignments};
use super::types::{RenterAssignPayload, RenterRow, RenterUpdatePayload, RenterWithUser};
use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{ApartmentRenter, NewApartmentRenter, PublicUser};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

/// List renters of an apartment
///
/// Returns all users who are registered as renters of the specified apartment.
/// Requires Admin, Manager role, or apartment ownership.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/renters",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of renters", body = Vec<RenterWithUser>),
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
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
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::users::dsl as users;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let user_id: u64 = auth
            .claims
            .sub
            .parse()
            .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }

    // Fetch apartment renters with manual field selection
    let renters_data: Vec<RenterRow> = ar::apartment_renters
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
/// Requires Admin, Manager role, or apartment ownership.
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
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
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
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(current_user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }
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
    let renter_data: RenterRow = ar::apartment_renters
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
/// Requires Admin, Manager role, or apartment ownership.
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
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
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
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;

    let (apartment_id, user_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(current_user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }
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
        diesel::update(target)
            .set(ar::start_date.eq(Some(start)))
            .execute(&mut conn)?;
    }

    if let Some(end) = payload.end_date {
        diesel::update(target)
            .set(ar::end_date.eq(Some(end)))
            .execute(&mut conn)?;
    }

    if let Some(active) = payload.is_active {
        diesel::update(target)
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
    let renter_data: RenterRow = ar::apartment_renters
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
/// Requires Admin, Manager role, or apartment ownership.
#[utoipa::path(
    delete,
    path = "/api/v1/apartments/{id}/renters/{user_id}",
    params(
        ("id" = u64, Path, description = "Apartment ID"),
        ("user_id" = u64, Path, description = "User ID to remove as renter")
    ),
    responses(
        (status = 204, description = "Renter removed successfully"),
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
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
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;

    let (apartment_id, user_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(current_user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }
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
