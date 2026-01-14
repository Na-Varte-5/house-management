use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::{Meter, MeterReading, MeterType};
use super::types::{CreateMeterRequest, UpdateMeterRequest, MeterWithLastReading};
use super::helpers::user_owns_apartment;

/// List meters for an apartment
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{apartment_id}/meters",
    params(
        ("apartment_id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of meters", body = Vec<MeterWithLastReading>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn list_apartment_meters(
    auth: AuthContext,
    apartment_id: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let apartment_id = apartment_id.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Check access: Admin/Manager can see all, others only if they own the apartment
    if !is_admin_or_manager && !user_owns_apartment(user_id, apartment_id, &mut conn)? {
        return Err(AppError::Forbidden);
    }

    use crate::schema::meters::dsl as m;

    let meters_list: Vec<Meter> = m::meters
        .filter(m::apartment_id.eq(apartment_id))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .load(&mut conn)?;

    // For each meter, get the last reading
    let mut result = Vec::new();
    for meter in meters_list {
        use crate::schema::meter_readings::dsl as mr;

        let last_reading: Option<MeterReading> = mr::meter_readings
            .filter(mr::meter_id.eq(meter.id))
            .order(mr::reading_timestamp.desc())
            .select(MeterReading::as_select())
            .first(&mut conn)
            .optional()?;

        let with_reading = MeterWithLastReading {
            meter,
            last_reading_value: last_reading.as_ref().map(|r| r.reading_value.to_string()),
            last_reading_timestamp: last_reading.as_ref().map(|r| r.reading_timestamp),
            last_reading_unit: last_reading.as_ref().map(|r| r.unit.clone()),
        };

        result.push(with_reading);
    }

    Ok(HttpResponse::Ok().json(result))
}

/// Get meter details
#[utoipa::path(
    get,
    path = "/api/v1/meters/{id}",
    params(
        ("id" = u64, Path, description = "Meter ID")
    ),
    responses(
        (status = 200, description = "Meter details", body = Meter),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn get_meter(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let meter_id = meter_id.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::meters::dsl as m;

    let meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)?;

    // Check access
    if !is_admin_or_manager && !user_owns_apartment(user_id, meter.apartment_id, &mut conn)? {
        return Err(AppError::Forbidden);
    }

    Ok(HttpResponse::Ok().json(meter))
}

/// Register new meter (Admin/Manager only)
#[utoipa::path(
    post,
    path = "/api/v1/meters",
    request_body = CreateMeterRequest,
    responses(
        (status = 201, description = "Meter created", body = Meter),
        (status = 403, description = "Forbidden"),
        (status = 400, description = "Bad request")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn create_meter(
    auth: AuthContext,
    payload: web::Json<CreateMeterRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    // Validate meter type
    let _: MeterType = payload.meter_type.parse()
        .map_err(|_| AppError::BadRequest("Invalid meter type".into()))?;

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::meters::dsl as m;

    // Parse dates
    let installation_date = payload.installation_date.as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let calibration_due_date = payload.calibration_due_date.as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    diesel::insert_into(m::meters)
        .values((
            m::apartment_id.eq(payload.apartment_id),
            m::meter_type.eq(&payload.meter_type),
            m::serial_number.eq(&payload.serial_number),
            m::is_visible_to_renters.eq(payload.is_visible_to_renters),
            m::installation_date.eq(installation_date),
            m::calibration_due_date.eq(calibration_due_date),
        ))
        .execute(&mut conn)?;

    let inserted_id: u64 = diesel::select(
        diesel::dsl::sql::<diesel::sql_types::Unsigned<diesel::sql_types::BigInt>>("LAST_INSERT_ID()")
    ).first(&mut conn)?;

    let meter: Meter = m::meters
        .filter(m::id.eq(inserted_id))
        .select(Meter::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Created().json(meter))
}

/// Update meter details (Admin/Manager only)
#[utoipa::path(
    put,
    path = "/api/v1/meters/{id}",
    params(
        ("id" = u64, Path, description = "Meter ID")
    ),
    request_body = UpdateMeterRequest,
    responses(
        (status = 200, description = "Meter updated", body = Meter),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn update_meter(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    payload: web::Json<UpdateMeterRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let meter_id = meter_id.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::meters::dsl as m;

    // Validate meter type if provided
    if let Some(ref mt) = payload.meter_type {
        let _: MeterType = mt.parse()
            .map_err(|_| AppError::BadRequest("Invalid meter type".into()))?;
    }

    // Update fields individually
    if let Some(ref mt) = payload.meter_type {
        diesel::update(m::meters.filter(m::id.eq(meter_id)))
            .set(m::meter_type.eq(mt))
            .execute(&mut conn)?;
    }
    if let Some(ref sn) = payload.serial_number {
        diesel::update(m::meters.filter(m::id.eq(meter_id)))
            .set(m::serial_number.eq(sn))
            .execute(&mut conn)?;
    }
    if let Some(vis) = payload.is_visible_to_renters {
        diesel::update(m::meters.filter(m::id.eq(meter_id)))
            .set(m::is_visible_to_renters.eq(vis))
            .execute(&mut conn)?;
    }
    if let Some(ref inst_date) = payload.installation_date {
        let date = chrono::NaiveDate::parse_from_str(inst_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid installation date format".into()))?;
        diesel::update(m::meters.filter(m::id.eq(meter_id)))
            .set(m::installation_date.eq(date))
            .execute(&mut conn)?;
    }
    if let Some(ref cal_date) = payload.calibration_due_date {
        let date = chrono::NaiveDate::parse_from_str(cal_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid calibration due date format".into()))?;
        diesel::update(m::meters.filter(m::id.eq(meter_id)))
            .set(m::calibration_due_date.eq(date))
            .execute(&mut conn)?;
    }

    let meter: Meter = m::meters
        .filter(m::id.eq(meter_id))
        .select(Meter::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(meter))
}

/// Deactivate meter (Admin/Manager only)
#[utoipa::path(
    delete,
    path = "/api/v1/meters/{id}",
    params(
        ("id" = u64, Path, description = "Meter ID")
    ),
    responses(
        (status = 204, description = "Meter deactivated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found")
    ),
    tag = "Meters",
    security(("bearer_auth" = []))
)]
pub async fn deactivate_meter(
    auth: AuthContext,
    meter_id: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    let meter_id = meter_id.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::meters::dsl as m;

    diesel::update(m::meters.filter(m::id.eq(meter_id)))
        .set(m::is_active.eq(false))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}
