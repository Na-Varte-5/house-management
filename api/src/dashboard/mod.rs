use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::{AppError, AuthContext, get_user_building_ids};
use crate::db::DbPool;

/// Dashboard statistics for the authenticated user
#[derive(Serialize, ToSchema)]
pub struct DashboardStats {
    /// Number of open maintenance requests (user created or assigned to, or all if Admin/Manager with building access)
    pub open_maintenance_count: i64,
    /// Number of active proposals (Open status, accessible to user)
    pub active_proposals_count: i64,
    /// Number of apartments the user owns or rents
    pub my_apartments_count: i64,
    /// Number of proposals the user is eligible to vote on but hasn't voted yet
    pub pending_votes_count: i64,
    /// Number of meters due for calibration within 30 days (if user has access to apartments with meters)
    pub meters_due_calibration: i64,
}

/// Get dashboard statistics
///
/// Returns statistics for the authenticated user including maintenance requests,
/// proposals, apartments, and meter calibration status.
#[utoipa::path(
    get,
    path = "/api/v1/dashboard",
    responses(
        (status = 200, description = "Dashboard statistics", body = DashboardStats),
        (status = 500, description = "Internal server error")
    ),
    tag = "Dashboard",
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard_stats(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin = auth.has_any_role(&["Admin"]);
    let is_manager = auth.has_any_role(&["Admin", "Manager"]);

    // Get accessible buildings for building-scoped filtering
    let building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    // Count open maintenance requests
    use crate::schema::apartments::dsl as apt;
    use crate::schema::maintenance_requests::dsl as mr;

    let maintenance_count = if is_admin {
        // Admin sees all open requests
        mr::maintenance_requests
            .filter(mr::status.eq("Open"))
            .count()
            .get_result::<i64>(&mut conn)?
    } else if is_manager {
        // Manager sees open requests from accessible buildings
        if let Some(ref ids) = building_ids {
            mr::maintenance_requests
                .inner_join(apt::apartments.on(apt::id.eq(mr::apartment_id)))
                .filter(mr::status.eq("Open"))
                .filter(apt::building_id.eq_any(ids))
                .count()
                .get_result::<i64>(&mut conn)?
        } else {
            0
        }
    } else {
        // Regular users see only their own requests or those assigned to them
        mr::maintenance_requests
            .filter(mr::status.eq("Open"))
            .filter(
                mr::created_by
                    .eq(user_id)
                    .or(mr::assigned_to.eq(Some(user_id))),
            )
            .count()
            .get_result::<i64>(&mut conn)?
    };

    // Get active proposals (Open status, accessible to user)
    use crate::schema::proposals::dsl as p;

    let mut proposals_query = p::proposals.filter(p::status.eq("Open")).into_boxed();

    if let Some(ref ids) = building_ids {
        proposals_query =
            proposals_query.filter(p::building_id.eq_any(ids).or(p::building_id.is_null()));
    }

    // Load proposal IDs first (for both count and pending votes)
    let active_proposal_ids: Vec<u64> = proposals_query.select(p::id).load(&mut conn)?;

    let active_proposals_count = active_proposal_ids.len() as i64;

    // Count apartments the user owns or rents
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;

    let owned_count: i64 = ao::apartment_owners
        .inner_join(apt::apartments.on(apt::id.eq(ao::apartment_id)))
        .filter(ao::user_id.eq(user_id))
        .filter(apt::is_deleted.eq(false))
        .count()
        .get_result(&mut conn)?;

    let rented_count: i64 = ar::apartment_renters
        .inner_join(apt::apartments.on(apt::id.eq(ar::apartment_id)))
        .filter(ar::user_id.eq(user_id))
        .filter(ar::is_active.eq(true))
        .filter(apt::is_deleted.eq(false))
        .count()
        .get_result(&mut conn)?;

    let my_apartments_count = owned_count + rented_count;

    // Count proposals user hasn't voted on yet
    use crate::schema::votes::dsl as v;

    let pending_votes_count = if active_proposal_ids.is_empty() {
        0
    } else {
        // Get proposals user has already voted on
        let voted_proposals: Vec<u64> = v::votes
            .filter(v::user_id.eq(user_id))
            .filter(v::proposal_id.eq_any(&active_proposal_ids))
            .select(v::proposal_id)
            .load(&mut conn)?;

        // Count proposals eligible but not voted
        active_proposal_ids.len() as i64 - voted_proposals.len() as i64
    };

    // Count meters due for calibration within 30 days
    use crate::schema::meters::dsl as m;

    let thirty_days_from_now =
        chrono::Local::now().naive_local().date() + chrono::Duration::days(30);

    let meters_due_calibration = if is_admin {
        // Admin sees all meters
        m::meters
            .filter(m::is_active.eq(true))
            .filter(m::calibration_due_date.is_not_null())
            .filter(m::calibration_due_date.le(thirty_days_from_now))
            .count()
            .get_result::<i64>(&mut conn)?
    } else {
        // Users see only meters from their accessible apartments
        // Get user's apartments (owned or rented)
        let user_apartments: Vec<u64> = if let Some(ref ids) = building_ids {
            apt::apartments
                .inner_join(ao::apartment_owners.on(ao::apartment_id.eq(apt::id)))
                .filter(ao::user_id.eq(user_id))
                .filter(apt::building_id.eq_any(ids))
                .filter(apt::is_deleted.eq(false))
                .select(apt::id)
                .union(
                    apt::apartments
                        .inner_join(ar::apartment_renters.on(ar::apartment_id.eq(apt::id)))
                        .filter(ar::user_id.eq(user_id))
                        .filter(ar::is_active.eq(true))
                        .filter(apt::building_id.eq_any(ids))
                        .filter(apt::is_deleted.eq(false))
                        .select(apt::id),
                )
                .load(&mut conn)?
        } else {
            vec![]
        };

        if user_apartments.is_empty() {
            0
        } else {
            m::meters
                .filter(m::is_active.eq(true))
                .filter(m::apartment_id.eq_any(&user_apartments))
                .filter(m::calibration_due_date.is_not_null())
                .filter(m::calibration_due_date.le(thirty_days_from_now))
                .count()
                .get_result::<i64>(&mut conn)?
        }
    };

    let stats = DashboardStats {
        open_maintenance_count: maintenance_count,
        active_proposals_count,
        my_apartments_count,
        pending_votes_count,
        meters_due_calibration,
    };

    Ok(HttpResponse::Ok().json(stats))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/dashboard", web::get().to(get_dashboard_stats));
}
