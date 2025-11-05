use crate::auth::error::AppError;
use crate::auth::types::{AuthResponse, Claims, JwtKeys, LoginRequest, RegisterRequest};
use crate::auth::{crypto, roles};
use crate::db::DbPool;
use crate::models::{NewUser, User};
use crate::schema::users;
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

pub async fn register(
    pool: web::Data<DbPool>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let password_hash = crypto::hash_password(&payload.password)?;
    let new_user = NewUser {
        email: payload.email.clone(),
        name: payload.name.clone(),
        password_hash,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)?;
    let created: User = users::table
        .filter(users::email.eq(&payload.email))
        .select(User::as_select())
        .first(&mut conn)?;
    let total = roles::count_users(&mut conn);
    let role_name = if total == 1 { "Admin" } else { "Homeowner" };
    if let Ok(role_id) = roles::ensure_role(role_name, &mut conn) {
        let _ = roles::assign_role(created.id, role_id, &mut conn);
    }
    Ok(HttpResponse::Created().finish())
}

pub async fn login(
    pool: web::Data<DbPool>,
    keys: web::Data<JwtKeys>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    use crate::schema::users::dsl as u;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user: User = u::users
        .filter(u::email.eq(&payload.email))
        .select(User::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::Unauthorized)?;
    if !crypto::verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }
    let roles_vec = roles::get_user_roles(user.id, &mut conn);
    let exp = chrono::Utc::now() + chrono::Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(), // include name in claims
        roles: roles_vec,
        exp: exp.timestamp() as usize,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &keys.encoding,
    )
    .map_err(|_| AppError::Internal("token_encode".into()))?;
    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    use actix_web::http::Method;
    cfg.service(
        web::resource("/auth/register")
            .route(web::post().to(register))
            .route(
                web::route()
                    .method(Method::OPTIONS)
                    .to(|| async { HttpResponse::NoContent().finish() }),
            ),
    )
    .service(
        web::resource("/auth/login")
            .route(web::post().to(login))
            .route(
                web::route()
                    .method(Method::OPTIONS)
                    .to(|| async { HttpResponse::NoContent().finish() }),
            ),
    );
}
