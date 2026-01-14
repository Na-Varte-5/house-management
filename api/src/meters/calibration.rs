use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::Meter;
use super::types::CalibrateMeterRequest;

/// List meters needing calibration (Admin/Manager only)
#[utoipa::path(
    get,
    path = "/api/v1/meters/calibration-due",
    params(
        ("days_before" = Option<i64>, Query, description = "Days before due date (default 30)")
    ),
    responses(
        (status = 200, description = "List of meters needing calibration", body = Vec<Meter>),
        (status = 403, description = "Forbidden")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn list_calibration_due(
    auth: AuthContext,
    query: web::Query<std::collections::HashMap<String, String>>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let days_before = query.get("days_before").and_then(|d| d.parse::<i64>().ok()).unwrap_or(30);
    let threshold_date = chrono::Utc::now().naive_utc().date() + chrono::Duration::days(days_before);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::meters::dsl as m;

    let meters_list: Vec<Meter> = m::meters
        .filter(m::is_active.eq(true))
        .filter(m::calibration_due_date.le(threshold_date))
        .select(Meter::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(meters_list))
}

/// Record meter calibration (Admin/Manager only)
#[utoipa::path(
    post,
    path = "/api/v1/meters/{id}/calibrate",
    params(
        ("id" = u64, Path, description = "Meter ID")
    ),
    request_body = CalibrateMeterRequest,
    responses(
        (status = 200, description = "Calibration recorded", body = Meter),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn calibrate_meter(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    payload: web::Json<CalibrateMeterRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let meter_id = meter_id.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Parse dates
    let cal_date = chrono::NaiveDate::parse_from_str(&payload.calibration_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid calibration date format".into()))?;
    let next_due = chrono::NaiveDate::parse_from_str(&payload.next_calibration_due, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid next calibration due date format".into()))?;

    use crate::schema::meters::dsl as m;

    diesel::update(m::meters.filter(m::id.eq(meter_id)))
        .set((
            m::last_calibration_date.eq(cal_date),
            m::calibration_due_date.eq(next_due),
        ))
        .execute(&mut conn)?;

    let meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .select(Meter::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(meter))
}
