use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::db::DbPool;
use crate::auth::{AuthContext, AppError};
use crate::models::WebhookApiKey;
use super::types::{CreateApiKeyRequest, ApiKeyResponse};

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
