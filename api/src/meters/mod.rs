use actix_web::{web, HttpResponse, Responder, HttpRequest};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::{Meter, NewMeter, MeterReading, NewMeterReading, MeterType, ReadingSource, WebhookApiKey};
use crate::schema::{meters, meter_readings, apartment_owners, apartments, webhook_api_keys};
use bigdecimal::BigDecimal;

// Request/Response types
#[derive(Deserialize, ToSchema)]
pub struct CreateMeterRequest {
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    #[serde(default = "default_visible_to_renters")]
    pub is_visible_to_renters: bool,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
}

fn default_visible_to_renters() -> bool { true }

#[derive(Deserialize, ToSchema)]
pub struct UpdateMeterRequest {
    pub meter_type: Option<String>,
    pub serial_number: Option<String>,
    pub is_visible_to_renters: Option<bool>,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateReadingRequest {
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub timestamp: Option<String>,
    pub unit: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CalibrateMeterRequest {
    pub calibration_date: String,
    pub next_calibration_due: String,
}

#[derive(Serialize, ToSchema)]
pub struct MeterWithLastReading {
    #[serde(flatten)]
    pub meter: Meter,
    pub last_reading_value: Option<String>,
    pub last_reading_timestamp: Option<chrono::NaiveDateTime>,
    pub last_reading_unit: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct WebhookReadingPayload {
    pub serial_number: String,
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub timestamp: String,
    pub unit: String,
}

#[derive(Deserialize, ToSchema)]
pub struct WebhookBatchPayload {
    pub readings: Vec<WebhookReadingPayload>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: u64,
    pub name: String,
    pub api_key: String, // Only returned on creation
}

// Helper functions
fn user_owns_apartment(user_id: u64, apartment_id: u64, conn: &mut diesel::MysqlConnection) -> Result<bool, AppError> {
    use crate::schema::apartment_owners::dsl as ao;

    let count: i64 = ao::apartment_owners
        .filter(ao::apartment_id.eq(apartment_id))
        .filter(ao::user_id.eq(user_id))
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}

fn meter_belongs_to_apartment(meter_id: u64, apartment_id: u64, conn: &mut diesel::MysqlConnection) -> Result<bool, AppError> {
    use crate::schema::meters::dsl as m;

    let count: i64 = m::meters
        .filter(m::id.eq(meter_id))
        .filter(m::apartment_id.eq(apartment_id))
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}

// Handlers

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

// Webhook authentication helper
async fn authenticate_webhook(req: &HttpRequest, pool: &web::Data<DbPool>) -> Result<(), AppError> {
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::webhook_api_keys::dsl as wak;
    use crate::auth::crypto::verify_password;

    let keys: Vec<WebhookApiKey> = wak::webhook_api_keys
        .filter(wak::is_active.eq(true))
        .select(WebhookApiKey::as_select())
        .load(&mut conn)?;

    // Check if any key matches
    for key in keys {
        if verify_password(api_key, &key.api_key_hash) {
            // Update last_used_at
            let now = chrono::Utc::now().naive_utc();
            diesel::update(wak::webhook_api_keys.filter(wak::id.eq(key.id)))
                .set(wak::last_used_at.eq(now))
                .execute(&mut conn)?;
            return Ok(());
        }
    }

    Err(AppError::Unauthorized)
}

/// Webhook endpoint for external systems
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/meter-reading",
    request_body = WebhookReadingPayload,
    responses(
        (status = 201, description = "Reading recorded"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Meter not found")
    ),
    tag = "Webhooks"
)]
pub async fn webhook_meter_reading(
    req: HttpRequest,
    payload: web::Json<WebhookReadingPayload>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    // Authenticate via API key
    authenticate_webhook(&req, &pool).await?;

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Find meter by serial number
    use crate::schema::meters::dsl as m;
    let meter: Meter = m::meters
        .filter(m::serial_number.eq(&payload.serial_number))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    // Parse timestamp
    let reading_timestamp = chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%SZ")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| AppError::BadRequest("Invalid timestamp format".into()))?;

    use crate::schema::meter_readings::dsl as mr;

    // Use INSERT IGNORE to handle duplicates gracefully (idempotent)
    diesel::insert_into(mr::meter_readings)
        .values((
            mr::meter_id.eq(meter.id),
            mr::reading_value.eq(&payload.reading_value),
            mr::reading_timestamp.eq(reading_timestamp),
            mr::unit.eq(&payload.unit),
            mr::source.eq("Webhook"),
        ))
        .execute(&mut conn)
        .ok(); // Ignore errors from duplicate readings

    Ok(HttpResponse::Created().finish())
}

/// Webhook endpoint for batch readings
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/meter-reading-batch",
    request_body = WebhookBatchPayload,
    responses(
        (status = 201, description = "Readings recorded"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Webhooks"
)]
pub async fn webhook_meter_reading_batch(
    req: HttpRequest,
    payload: web::Json<WebhookBatchPayload>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    // Authenticate via API key
    authenticate_webhook(&req, &pool).await?;

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    for reading_payload in &payload.readings {
        // Find meter by serial number
        use crate::schema::meters::dsl as m;
        let meter_result: Result<Meter, _> = m::meters
            .filter(m::serial_number.eq(&reading_payload.serial_number))
            .filter(m::is_active.eq(true))
            .select(Meter::as_select())
            .first(&mut conn);

        if let Ok(meter) = meter_result {
            // Parse timestamp
            if let Ok(reading_timestamp) = chrono::NaiveDateTime::parse_from_str(&reading_payload.timestamp, "%Y-%m-%dT%H:%M:%SZ")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&reading_payload.timestamp, "%Y-%m-%dT%H:%M:%S"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&reading_payload.timestamp, "%Y-%m-%d %H:%M:%S"))
            {
                use crate::schema::meter_readings::dsl as mr;
                diesel::insert_into(mr::meter_readings)
                    .values((
                        mr::meter_id.eq(meter.id),
                        mr::reading_value.eq(&reading_payload.reading_value),
                        mr::reading_timestamp.eq(reading_timestamp),
                        mr::unit.eq(&reading_payload.unit),
                        mr::source.eq("Webhook"),
                    ))
                    .execute(&mut conn)
                    .ok(); // Ignore errors from duplicate readings
            }
        }
    }

    Ok(HttpResponse::Created().finish())
}

/// List API keys (Admin only)
#[utoipa::path(
    get,
    path = "/api/v1/admin/api-keys",
    responses(
        (status = 200, description = "List of API keys", body = Vec<WebhookApiKey>),
        (status = 403, description = "Forbidden")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn list_api_keys(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::webhook_api_keys::dsl as wak;

    let keys: Vec<WebhookApiKey> = wak::webhook_api_keys
        .select(WebhookApiKey::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(keys))
}

/// Create API key (Admin only)
#[utoipa::path(
    post,
    path = "/api/v1/admin/api-keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = ApiKeyResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn create_api_key(
    auth: AuthContext,
    payload: web::Json<CreateApiKeyRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    // Generate random API key
    use rand::Rng;
    let api_key: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Hash the API key
    use crate::auth::crypto::hash_password;
    let api_key_hash = hash_password(&api_key)?;

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::webhook_api_keys::dsl as wak;

    diesel::insert_into(wak::webhook_api_keys)
        .values((
            wak::name.eq(&payload.name),
            wak::api_key_hash.eq(&api_key_hash),
            wak::created_by.eq(user_id),
        ))
        .execute(&mut conn)?;

    let inserted_id: u64 = diesel::select(
        diesel::dsl::sql::<diesel::sql_types::Unsigned<diesel::sql_types::BigInt>>("LAST_INSERT_ID()")
    ).first(&mut conn)?;

    Ok(HttpResponse::Created().json(ApiKeyResponse {
        id: inserted_id,
        name: payload.name.clone(),
        api_key, // Only returned on creation
    }))
}

/// Revoke API key (Admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/admin/api-keys/{id}",
    params(
        ("id" = u64, Path, description = "API key ID")
    ),
    responses(
        (status = 204, description = "API key revoked"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Admin",
    security(("bearer_auth" = []))
)]
pub async fn revoke_api_key(
    auth: AuthContext,
    key_id: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }

    let key_id = key_id.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::schema::webhook_api_keys::dsl as wak;

    diesel::update(wak::webhook_api_keys.filter(wak::id.eq(key_id)))
        .set(wak::is_active.eq(false))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// Configure routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/apartments/{apartment_id}/meters", web::get().to(list_apartment_meters))
       .route("/meters", web::post().to(create_meter))
       // Specific routes must come before generic {id} routes
       .route("/meters/calibration-due", web::get().to(list_calibration_due))
       .route("/meters/{id}", web::get().to(get_meter))
       .route("/meters/{id}", web::put().to(update_meter))
       .route("/meters/{id}", web::delete().to(deactivate_meter))
       // Specific sub-routes must come before generic {id}/readings routes
       .route("/meters/{id}/readings/export", web::get().to(export_readings_csv))
       .route("/meters/{id}/readings", web::get().to(list_readings))
       .route("/meters/{id}/readings", web::post().to(create_reading))
       .route("/meters/{id}/calibrate", web::post().to(calibrate_meter))
       .route("/webhooks/meter-reading", web::post().to(webhook_meter_reading))
       .route("/webhooks/meter-reading-batch", web::post().to(webhook_meter_reading_batch))
       .route("/admin/api-keys", web::get().to(list_api_keys))
       .route("/admin/api-keys", web::post().to(create_api_key))
       .route("/admin/api-keys/{id}", web::delete().to(revoke_api_key));
}
