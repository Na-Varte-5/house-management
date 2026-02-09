use super::types::{
    AssignPayload, EscalatePayload, MaintenanceRequestDetail, MaintenanceRequestDetailRow,
    MaintenanceRequestEnriched, MaintenanceRequestHistoryEnriched, MaintenanceRequestQueryRow,
    StatusUpdatePayload, UpdateRequestPayload,
};
use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{MaintenanceRequest, MaintenanceRequestHistory, NewMaintenanceRequest};
use crate::pagination::{PaginatedResponse, PaginationParams};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

/// List maintenance requests
///
/// Returns maintenance requests with enriched data (apartment number and building address).
/// Based on user role:
/// - Admin/Manager: See all requests
/// - Others: See only requests they created or are assigned to
#[utoipa::path(
    get,
    path = "/api/v1/requests",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of maintenance requests with enriched data", body = PaginatedResponse<MaintenanceRequestEnriched>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn list_requests(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, AppError> {
    use crate::auth::get_user_building_ids;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::maintenance_requests::dsl as mr;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id = auth.user_id()?;
    let is_admin = auth.has_any_role(&["Admin"]);
    let is_manager = auth.has_any_role(&["Manager"]);

    let building_ids = if !is_admin {
        get_user_building_ids(user_id, is_admin, &mut conn)?
    } else {
        None
    };

    let mut count_query = mr::maintenance_requests
        .inner_join(apt::apartments.on(apt::id.eq(mr::apartment_id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .into_boxed();

    if !is_admin {
        if is_manager {
            if let Some(ref ids) = building_ids {
                count_query = count_query.filter(apt::building_id.eq_any(ids));
            }
        } else {
            count_query = count_query.filter(
                mr::created_by
                    .eq(user_id)
                    .or(mr::assigned_to.eq(Some(user_id))),
            );
        }
    }

    let total = count_query.count().get_result::<i64>(&mut conn)?;

    let mut data_query = mr::maintenance_requests
        .inner_join(apt::apartments.on(apt::id.eq(mr::apartment_id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .into_boxed();

    if !is_admin {
        if is_manager {
            if let Some(ref ids) = building_ids {
                data_query = data_query.filter(apt::building_id.eq_any(ids));
            }
        } else {
            data_query = data_query.filter(
                mr::created_by
                    .eq(user_id)
                    .or(mr::assigned_to.eq(Some(user_id))),
            );
        }
    }

    let results: Vec<MaintenanceRequestQueryRow> = data_query
        .select((
            mr::id,
            mr::apartment_id,
            mr::created_by,
            mr::assigned_to,
            mr::request_type,
            mr::priority,
            mr::title,
            mr::description,
            mr::status,
            mr::created_at,
            apt::number,
            apt::building_id,
            bld::address,
        ))
        .limit(query.limit())
        .offset(query.offset())
        .load(&mut conn)?;

    let enriched: Vec<MaintenanceRequestEnriched> = results
        .into_iter()
        .map(
            |(
                id,
                apartment_id,
                created_by,
                assigned_to,
                request_type,
                priority,
                title,
                description,
                status,
                created_at,
                apt_number,
                bld_id,
                bld_addr,
            )| MaintenanceRequestEnriched {
                id,
                apartment_id,
                apartment_number: apt_number,
                building_id: bld_id,
                building_address: bld_addr,
                request_type,
                priority,
                title,
                description,
                status,
                created_by,
                assigned_to,
                created_at: created_at
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
            },
        )
        .collect();

    Ok(HttpResponse::Ok().json(PaginatedResponse::new(enriched, total, &query)))
}

/// Get a single maintenance request
///
/// Returns details of a specific maintenance request with enriched data (apartment, building, and user names).
/// Users can only view requests they created, are assigned to, or all requests if they are Admin/Manager.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "Maintenance request details with enriched data", body = MaintenanceRequestDetail),
        (status = 403, description = "Forbidden - cannot view this request"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn get_request(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as usr;

    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let result: MaintenanceRequestDetailRow = mr::maintenance_requests
        .inner_join(apt::apartments.on(apt::id.eq(mr::apartment_id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .inner_join(usr::users.on(usr::id.eq(mr::created_by)))
        .filter(mr::id.eq(id))
        .select((
            mr::id,
            mr::apartment_id,
            mr::created_by,
            mr::assigned_to,
            mr::request_type,
            mr::priority,
            mr::title,
            mr::description,
            mr::status,
            mr::resolution_notes,
            mr::created_at,
            mr::updated_at,
            apt::number,
            apt::building_id,
            bld::address,
            usr::name,
        ))
        .first(&mut conn)?;

    let (
        req_id,
        apartment_id,
        created_by,
        assigned_to,
        request_type,
        priority,
        title,
        description,
        status,
        resolution_notes,
        created_at,
        updated_at,
        apt_number,
        bld_id,
        bld_addr,
        creator_name,
    ) = result;

    let user_id = auth.user_id()?;
    if !(auth.has_any_role(&["Admin", "Manager"])
        || created_by == user_id
        || assigned_to == Some(user_id))
    {
        return Err(AppError::Forbidden);
    }

    let assigned_name = if let Some(assigned_id) = assigned_to {
        usr::users
            .filter(usr::id.eq(assigned_id))
            .select(usr::name)
            .first::<String>(&mut conn)
            .ok()
    } else {
        None
    };

    let detail = MaintenanceRequestDetail {
        id: req_id,
        apartment_id,
        apartment_number: apt_number,
        building_id: bld_id,
        building_address: bld_addr,
        request_type,
        priority,
        title,
        description,
        status,
        resolution_notes,
        created_by,
        created_by_name: creator_name,
        assigned_to,
        assigned_to_name: assigned_name,
        created_at: created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
        updated_at: updated_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(detail))
}

/// Create a maintenance request
///
/// Creates a new maintenance request for an apartment. Requires Homeowner, Renter,
/// Admin, or Manager role. The request is created with "Open" status.
/// Returns the ID of the created request.
#[utoipa::path(
    post,
    path = "/api/v1/requests",
    request_body = NewMaintenanceRequest,
    responses(
        (status = 201, description = "Request created successfully", body = inline(Object), example = json!({"id": 1})),
        (status = 403, description = "Forbidden - requires Homeowner, Renter, Admin, or Manager role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn create_request(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    payload: web::Json<NewMaintenanceRequest>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_requests::dsl as mr;
    if !auth.has_any_role(&["Homeowner", "Renter", "Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let new = payload.into_inner();
    diesel::insert_into(mr::maintenance_requests)
        .values((
            mr::apartment_id.eq(new.apartment_id),
            mr::created_by.eq(auth.user_id()?),
            mr::request_type.eq(new.request_type),
            mr::priority.eq(new.priority),
            mr::title.eq(new.title),
            mr::description.eq(new.description),
            mr::status.eq("Open"),
        ))
        .execute(&mut conn)?;

    let inserted_id: u64 = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
    >("LAST_INSERT_ID()"))
    .first(&mut conn)?;

    #[derive(serde::Serialize)]
    struct CreatedResponse {
        id: u64,
    }

    Ok(HttpResponse::Created().json(CreatedResponse { id: inserted_id }))
}

/// Update request status with audit trail
///
/// Updates the status of a maintenance request and records the change in the history table.
/// Requires Admin or Manager role.
#[utoipa::path(
    put,
    path = "/api/v1/requests/{id}/status",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body = StatusUpdatePayload,
    responses(
        (status = 200, description = "Status updated successfully"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn update_status(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<StatusUpdatePayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let user_id = auth
        .claims
        .sub
        .parse::<u64>()
        .map_err(|_| AppError::Internal("invalid_user_id_in_jwt".into()))?;

    let user_exists = u::users
        .filter(u::id.eq(user_id))
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;
    if !user_exists {
        return Err(AppError::Internal("user_not_found_in_database".into()));
    }

    let current: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;
    let new_status = payload.status.clone();
    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
        .set((
            mr::status.eq(&new_status),
            mr::resolution_notes.eq(payload.note.clone()),
        ))
        .execute(&mut conn)?;
    diesel::insert_into(hist::maintenance_request_history)
        .values((
            hist::request_id.eq(id),
            hist::from_status.eq(current.status),
            hist::to_status.eq(new_status),
            hist::note.eq(payload.note.clone()),
            hist::changed_by.eq(user_id),
        ))
        .execute(&mut conn)?;
    Ok(HttpResponse::Ok().finish())
}

/// Update maintenance request fields
///
/// General update endpoint that allows updating status, priority, and/or assignment.
/// Status changes are recorded in the history table. Requires Admin or Manager role.
#[utoipa::path(
    put,
    path = "/api/v1/requests/{id}",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body = UpdateRequestPayload,
    responses(
        (status = 200, description = "Request updated successfully with enriched data", body = MaintenanceRequestDetail),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn update_request(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<UpdateRequestPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let user_id = auth
        .claims
        .sub
        .parse::<u64>()
        .map_err(|_| AppError::Internal("invalid_user_id_in_jwt".into()))?;

    let user_exists = u::users
        .filter(u::id.eq(user_id))
        .count()
        .get_result::<i64>(&mut conn)?
        > 0;
    if !user_exists {
        return Err(AppError::Internal("user_not_found_in_database".into()));
    }

    let current: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;
    let current_status = current.status.clone();

    if let Some(new_status) = &payload.status {
        diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
            .set(mr::status.eq(new_status))
            .execute(&mut conn)?;
        diesel::insert_into(hist::maintenance_request_history)
            .values((
                hist::request_id.eq(id),
                hist::from_status.eq(&current.status),
                hist::to_status.eq(new_status),
                hist::note.eq::<Option<String>>(None),
                hist::changed_by.eq(user_id),
            ))
            .execute(&mut conn)?;
    }

    if let Some(new_priority) = &payload.priority {
        diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
            .set(mr::priority.eq(new_priority))
            .execute(&mut conn)?;

        let note = format!(
            "Priority changed from {} to {}",
            current.priority, new_priority
        );
        diesel::insert_into(hist::maintenance_request_history)
            .values((
                hist::request_id.eq(id),
                hist::from_status.eq::<Option<String>>(None),
                hist::to_status.eq(&current_status),
                hist::note.eq(Some(note)),
                hist::changed_by.eq(user_id),
            ))
            .execute(&mut conn)?;
    }

    if let Some(new_assigned) = payload.assigned_to {
        let new_assignee_name: String = u::users
            .filter(u::id.eq(new_assigned))
            .select(u::name)
            .first(&mut conn)
            .unwrap_or_else(|_| format!("User #{}", new_assigned));

        diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
            .set(mr::assigned_to.eq(Some(new_assigned)))
            .execute(&mut conn)?;

        let note = if let Some(prev_assigned) = current.assigned_to {
            let prev_assignee_name: String = u::users
                .filter(u::id.eq(prev_assigned))
                .select(u::name)
                .first(&mut conn)
                .unwrap_or_else(|_| format!("User #{}", prev_assigned));
            format!(
                "Reassigned from {} to {}",
                prev_assignee_name, new_assignee_name
            )
        } else {
            format!("Assigned to {}", new_assignee_name)
        };

        diesel::insert_into(hist::maintenance_request_history)
            .values((
                hist::request_id.eq(id),
                hist::from_status.eq::<Option<String>>(None),
                hist::to_status.eq(&current_status),
                hist::note.eq(Some(note)),
                hist::changed_by.eq(user_id),
            ))
            .execute(&mut conn)?;
    }

    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::users::dsl as usr;

    let result: MaintenanceRequestDetailRow = mr::maintenance_requests
        .inner_join(apt::apartments.on(apt::id.eq(mr::apartment_id)))
        .inner_join(bld::buildings.on(bld::id.eq(apt::building_id)))
        .inner_join(usr::users.on(usr::id.eq(mr::created_by)))
        .filter(mr::id.eq(id))
        .select((
            mr::id,
            mr::apartment_id,
            mr::created_by,
            mr::assigned_to,
            mr::request_type,
            mr::priority,
            mr::title,
            mr::description,
            mr::status,
            mr::resolution_notes,
            mr::created_at,
            mr::updated_at,
            apt::number,
            apt::building_id,
            bld::address,
            usr::name,
        ))
        .first(&mut conn)?;

    let (
        req_id,
        apartment_id,
        created_by,
        assigned_to,
        request_type,
        priority,
        title,
        description,
        status,
        resolution_notes,
        created_at,
        updated_at,
        apt_number,
        bld_id,
        bld_addr,
        creator_name,
    ) = result;

    let assigned_name = if let Some(assigned_id) = assigned_to {
        usr::users
            .filter(usr::id.eq(assigned_id))
            .select(usr::name)
            .first::<String>(&mut conn)
            .ok()
    } else {
        None
    };

    let detail = MaintenanceRequestDetail {
        id: req_id,
        apartment_id,
        apartment_number: apt_number,
        building_id: bld_id,
        building_address: bld_addr,
        request_type,
        priority,
        title,
        description,
        status,
        resolution_notes,
        created_by,
        created_by_name: creator_name,
        assigned_to,
        assigned_to_name: assigned_name,
        created_at: created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
        updated_at: updated_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(detail))
}

/// List history entries for a request
///
/// Returns the audit trail of status changes for a maintenance request with enriched data (user names).
/// Accessible by Admin, Manager, or the request creator.
#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/history",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "List of history entries with user names", body = Vec<MaintenanceRequestHistoryEnriched>),
        (status = 403, description = "Forbidden - can only view history of own requests or if Admin/Manager"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn list_history(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as usr;

    let request_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let req: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(request_id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;
    let user_id = auth.user_id()?;
    if !(auth.has_any_role(&["Admin", "Manager"]) || req.created_by == user_id) {
        return Err(AppError::Forbidden);
    }

    let results: Vec<(MaintenanceRequestHistory, String)> = hist::maintenance_request_history
        .inner_join(usr::users.on(usr::id.eq(hist::changed_by)))
        .filter(hist::request_id.eq(request_id))
        .select((MaintenanceRequestHistory::as_select(), usr::name))
        .order(hist::changed_at.asc())
        .load(&mut conn)?;

    let enriched: Vec<MaintenanceRequestHistoryEnriched> = results
        .into_iter()
        .map(|(entry, user_name)| MaintenanceRequestHistoryEnriched {
            id: entry.id,
            request_id: entry.request_id,
            from_status: entry.from_status,
            to_status: entry.to_status,
            note: entry.note,
            changed_by: entry.changed_by,
            changed_by_name: user_name,
            changed_at: entry
                .changed_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(enriched))
}

/// Assign a maintenance request to a user
///
/// Assigns a maintenance request to a specific user. Verifies the user exists.
/// Requires Admin or Manager role.
#[utoipa::path(
    put,
    path = "/api/v1/requests/{id}/assign",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body = AssignPayload,
    responses(
        (status = 200, description = "Request assigned successfully", body = MaintenanceRequest),
        (status = 400, description = "Bad request - user not found"),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn assign_request(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<AssignPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let target_user = payload.user_id;
    let user_id = auth.user_id()?;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let exists: Result<u64, _> = u::users
        .filter(u::id.eq(target_user))
        .select(u::id)
        .first(&mut conn);
    if exists.is_err() {
        return Err(AppError::BadRequest("user_not_found".into()));
    }

    let current: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;

    let old_assigned = current.assigned_to;

    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
        .set(mr::assigned_to.eq(Some(target_user)))
        .execute(&mut conn)?;

    let new_name: String = u::users
        .filter(u::id.eq(target_user))
        .select(u::name)
        .first(&mut conn)
        .unwrap_or_else(|_| format!("User {}", target_user));

    let note = if let Some(old_id) = old_assigned {
        let old_name: String = u::users
            .filter(u::id.eq(old_id))
            .select(u::name)
            .first(&mut conn)
            .unwrap_or_else(|_| format!("User {}", old_id));
        format!("Reassigned from {} to {}", old_name, new_name)
    } else {
        format!("Assigned to {}", new_name)
    };

    diesel::insert_into(hist::maintenance_request_history)
        .values((
            hist::request_id.eq(id),
            hist::from_status.eq::<Option<String>>(None),
            hist::to_status.eq(&current.status),
            hist::note.eq(Some(note)),
            hist::changed_by.eq(user_id),
        ))
        .execute(&mut conn)?;

    let updated: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(updated))
}

/// Unassign a maintenance request
///
/// Removes the assignment from a maintenance request (sets assigned_to to NULL).
/// Requires Admin or Manager role.
#[utoipa::path(
    delete,
    path = "/api/v1/requests/{id}/assign",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    responses(
        (status = 200, description = "Request unassigned successfully", body = MaintenanceRequest),
        (status = 403, description = "Forbidden - requires Admin or Manager role"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn unassign_request(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    let id = path.into_inner();
    let user_id = auth.user_id()?;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;

    let old_assigned = current.assigned_to;

    diesel::update(mr::maintenance_requests.filter(mr::id.eq(id)))
        .set(mr::assigned_to.eq::<Option<u64>>(None))
        .execute(&mut conn)?;

    if let Some(old_id) = old_assigned {
        let old_name: String = u::users
            .filter(u::id.eq(old_id))
            .select(u::name)
            .first(&mut conn)
            .unwrap_or_else(|_| format!("User {}", old_id));

        let note = format!("Unassigned from {}", old_name);

        diesel::insert_into(hist::maintenance_request_history)
            .values((
                hist::request_id.eq(id),
                hist::from_status.eq::<Option<String>>(None),
                hist::to_status.eq(&current.status),
                hist::note.eq(Some(note)),
                hist::changed_by.eq(user_id),
            ))
            .execute(&mut conn)?;
    }

    let updated: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(updated))
}

/// Escalate a maintenance request to a building manager
///
/// Allows apartment owners to escalate a maintenance request by reassigning it to a building manager.
/// The user must own the apartment associated with the request.
/// The target manager must be a manager of the building where the apartment is located.
#[utoipa::path(
    post,
    path = "/api/v1/requests/{id}/escalate",
    params(
        ("id" = u64, Path, description = "Maintenance request ID")
    ),
    request_body = EscalatePayload,
    responses(
        (status = 200, description = "Request escalated successfully"),
        (status = 400, description = "Bad request - invalid manager or not a manager of this building"),
        (status = 403, description = "Forbidden - not an owner of this apartment"),
        (status = 404, description = "Request not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Maintenance",
    security(("bearer_auth" = []))
)]
pub async fn escalate_request(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<EscalatePayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::building_managers::dsl as bm;
    use crate::schema::maintenance_request_history::dsl as hist;
    use crate::schema::maintenance_requests::dsl as mr;
    use crate::schema::users::dsl as u;

    let request_id = path.into_inner();
    let target_manager_id = payload.manager_id;
    let user_id = auth.user_id()?;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let request: MaintenanceRequest = mr::maintenance_requests
        .filter(mr::id.eq(request_id))
        .select(MaintenanceRequest::as_select())
        .first(&mut conn)?;

    let apartment: u64 = apt::apartments
        .filter(apt::id.eq(request.apartment_id))
        .select(apt::building_id)
        .first(&mut conn)?;
    let building_id = apartment;

    let is_owner: bool = ao::apartment_owners
        .filter(
            ao::apartment_id
                .eq(request.apartment_id)
                .and(ao::user_id.eq(user_id)),
        )
        .select((ao::apartment_id, ao::user_id))
        .first::<(u64, u64)>(&mut conn)
        .is_ok();

    let is_admin_mgr = auth.has_any_role(&["Admin", "Manager"]);

    if !is_owner && !is_admin_mgr {
        return Err(AppError::Forbidden);
    }

    let is_valid_manager: bool = bm::building_managers
        .filter(
            bm::building_id
                .eq(building_id)
                .and(bm::user_id.eq(target_manager_id)),
        )
        .select((bm::building_id, bm::user_id))
        .first::<(u64, u64)>(&mut conn)
        .is_ok();

    if !is_valid_manager {
        return Err(AppError::BadRequest(
            "Target user is not a manager of this building".into(),
        ));
    }

    let old_assigned = request.assigned_to;

    diesel::update(mr::maintenance_requests.filter(mr::id.eq(request_id)))
        .set(mr::assigned_to.eq(Some(target_manager_id)))
        .execute(&mut conn)?;

    let manager_name: String = u::users
        .filter(u::id.eq(target_manager_id))
        .select(u::name)
        .first(&mut conn)
        .unwrap_or_else(|_| format!("User {}", target_manager_id));

    let note = if let Some(old_id) = old_assigned {
        let old_name: String = u::users
            .filter(u::id.eq(old_id))
            .select(u::name)
            .first(&mut conn)
            .unwrap_or_else(|_| format!("User {}", old_id));
        format!("Escalated from {} to {}", old_name, manager_name)
    } else {
        format!("Escalated to {}", manager_name)
    };

    diesel::insert_into(hist::maintenance_request_history)
        .values((
            hist::request_id.eq(request_id),
            hist::from_status.eq::<Option<String>>(None),
            hist::to_status.eq(&request.status),
            hist::note.eq(Some(note)),
            hist::changed_by.eq(user_id),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Request escalated successfully"})))
}
