use super::types::{WebhookBatchPayload, WebhookReadingPayload};
use crate::auth::AppError;
use crate::db::DbPool;
use crate::models::{Meter, WebhookApiKey};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use diesel::prelude::*;

/// Authenticate webhook requests via API key
async fn authenticate_webhook(req: &HttpRequest, pool: &web::Data<DbPool>) -> Result<(), AppError> {
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    use crate::auth::crypto::verify_password;
    use crate::schema::webhook_api_keys::dsl as wak;

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

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    // Find meter by serial number
    use crate::schema::meters::dsl as m;
    let meter: Meter = m::meters
        .filter(m::serial_number.eq(&payload.serial_number))
        .filter(m::is_active.eq(true))
        .select(Meter::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    // Parse timestamp
    let reading_timestamp =
        chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%SZ")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%dT%H:%M:%S")
            })
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&payload.timestamp, "%Y-%m-%d %H:%M:%S")
            })
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

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

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
            if let Ok(reading_timestamp) = chrono::NaiveDateTime::parse_from_str(
                &reading_payload.timestamp,
                "%Y-%m-%dT%H:%M:%SZ",
            )
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(
                    &reading_payload.timestamp,
                    "%Y-%m-%dT%H:%M:%S",
                )
            })
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(
                    &reading_payload.timestamp,
                    "%Y-%m-%d %H:%M:%S",
                )
            }) {
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
