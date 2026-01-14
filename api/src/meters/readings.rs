use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::{Meter, MeterReading};
use super::types::CreateReadingRequest;
use super::helpers::user_owns_apartment;

/// Get historical readings for a meter
#[utoipa::path(
    get,
    path = "/api/v1/meters/{id}/readings",
    params(
        ("id" = u64, Path, description = "Meter ID"),
        ("start_date" = Option<String>, Query, description = "Start date (YYYY-MM-DD)"),
        ("end_date" = Option<String>, Query, description = "End date (YYYY-MM-DD)"),
        ("page" = Option<u64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<u64>, Query, description = "Items per page (default 50)")
    ),
    responses(
        (status = 200, description = "List of readings", body = Vec<MeterReading>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn list_readings(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    query: web::Query<std::collections::HashMap<String, String>>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let meter_id = meter_id.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get meter and check access
    use crate::schema::meters::dsl as m;
    let meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)?;

    if !is_admin_or_manager && !user_owns_apartment(user_id, meter.apartment_id, &mut conn)? {
        return Err(AppError::Forbidden);
    }

    // Parse query parameters
    let page = query.get("page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(1);
    let per_page = query.get("per_page").and_then(|p| p.parse::<i64>().ok()).unwrap_or(50);
    let offset = (page - 1) * per_page;

    use crate::schema::meter_readings::dsl as mr;
    let mut query_builder = mr::meter_readings
        .filter(mr::meter_id.eq(meter_id))
        .into_boxed();

    // Date filtering
    if let Some(start_date_str) = query.get("start_date") {
        if let Ok(start_date) = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d") {
            let start_datetime = start_date.and_hms_opt(0, 0, 0).unwrap();
            query_builder = query_builder.filter(mr::reading_timestamp.ge(start_datetime));
        }
    }
    if let Some(end_date_str) = query.get("end_date") {
        if let Ok(end_date) = chrono::NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d") {
            let end_datetime = end_date.and_hms_opt(23, 59, 59).unwrap();
            query_builder = query_builder.filter(mr::reading_timestamp.le(end_datetime));
        }
    }

    let readings: Vec<MeterReading> = query_builder
        .order(mr::reading_timestamp.desc())
        .limit(per_page)
        .offset(offset)
        .select(MeterReading::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(readings))
}

/// Manual reading entry (Admin/Manager only)
#[utoipa::path(
    post,
    path = "/api/v1/meters/{id}/readings",
    params(
        ("id" = u64, Path, description = "Meter ID")
    ),
    request_body = CreateReadingRequest,
    responses(
        (status = 201, description = "Reading created", body = MeterReading),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn create_reading(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    payload: web::Json<CreateReadingRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let meter_id = meter_id.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Verify meter exists
    use crate::schema::meters::dsl as m;
    let _meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)?;

    // Parse timestamp
    let reading_timestamp = if let Some(ref ts_str) = payload.timestamp {
        chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S"))
            .map_err(|_| AppError::BadRequest("Invalid timestamp format".into()))?
    } else {
        chrono::Utc::now().naive_utc()
    };

    use crate::schema::meter_readings::dsl as mr;

    diesel::insert_into(mr::meter_readings)
        .values((
            mr::meter_id.eq(meter_id),
            mr::reading_value.eq(&payload.reading_value),
            mr::reading_timestamp.eq(reading_timestamp),
            mr::unit.eq(&payload.unit),
            mr::source.eq("Manual"),
        ))
        .execute(&mut conn)?;

    let inserted_id: u64 = diesel::select(
        diesel::dsl::sql::<diesel::sql_types::Unsigned<diesel::sql_types::BigInt>>("LAST_INSERT_ID()")
    ).first(&mut conn)?;

    let reading: MeterReading = mr::meter_readings
        .filter(mr::id.eq(inserted_id))
        .select(MeterReading::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Created().json(reading))
}

/// Export readings as CSV
#[utoipa::path(
    get,
    path = "/api/v1/meters/{id}/readings/export",
    params(
        ("id" = u64, Path, description = "Meter ID"),
        ("start_date" = Option<String>, Query, description = "Start date (YYYY-MM-DD)"),
        ("end_date" = Option<String>, Query, description = "End date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "CSV export", content_type = "text/csv"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn export_readings_csv(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    query: web::Query<std::collections::HashMap<String, String>>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let meter_id = meter_id.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Get meter and check access
    use crate::schema::meters::dsl as m;
    let meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)?;

    if !is_admin_or_manager && !user_owns_apartment(user_id, meter.apartment_id, &mut conn)? {
        return Err(AppError::Forbidden);
    }

    use crate::schema::meter_readings::dsl as mr;
    let mut query_builder = mr::meter_readings
        .filter(mr::meter_id.eq(meter_id))
        .into_boxed();

    // Date filtering
    if let Some(start_date_str) = query.get("start_date") {
        if let Ok(start_date) = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d") {
            let start_datetime = start_date.and_hms_opt(0, 0, 0).unwrap();
            query_builder = query_builder.filter(mr::reading_timestamp.ge(start_datetime));
        }
    }
    if let Some(end_date_str) = query.get("end_date") {
        if let Ok(end_date) = chrono::NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d") {
            let end_datetime = end_date.and_hms_opt(23, 59, 59).unwrap();
            query_builder = query_builder.filter(mr::reading_timestamp.le(end_datetime));
        }
    }

    let readings: Vec<MeterReading> = query_builder
        .order(mr::reading_timestamp.asc())
        .select(MeterReading::as_select())
        .load(&mut conn)?;

    // Generate CSV
    let mut csv_content = "Timestamp,Value,Unit,Source\n".to_string();
    for reading in readings {
        csv_content.push_str(&format!(
            "{},{},{},{}\n",
            reading.reading_timestamp,
            reading.reading_value,
            reading.unit,
            reading.source
        ));
    }

    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"meter_{}_readings.csv\"", meter_id)))
        .body(csv_content))
}
