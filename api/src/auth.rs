use crate::models::{NewUser, User};
use crate::schema::{roles, user_roles, users};
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use chrono::{Duration, Utc};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn from_secret(secret: &str) -> Self {
        JwtKeys {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String, // user id as string
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<MysqlConnection>>;

fn hash_password(password: &str) -> Result<String, HttpResponse> {
    use argon2::password_hash::{Error as PwdHashError, SaltString};
    use argon2::{Argon2, PasswordHasher};
    use rand::thread_rng;

    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|ph| ph.to_string())
        .map_err(|_e: PwdHashError| HttpResponse::InternalServerError().finish())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};

    if let Ok(parsed) = PasswordHash::new(password_hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    } else {
        false
    }
}

fn get_user_roles(user_id: u64, conn: &mut MysqlConnection) -> Vec<String> {
    use roles::dsl as r;
    use user_roles::dsl as ur;

    let res = ur::user_roles
        .inner_join(r::roles.on(r::id.eq(ur::role_id)))
        .filter(ur::user_id.eq(user_id))
        .select(r::name)
        .load::<String>(conn);

    res.unwrap_or_default()
}

fn count_users(conn: &mut MysqlConnection) -> i64 {
    users::table
        .count()
        .get_result::<i64>(conn)
        .unwrap_or_default()
}

fn ensure_role(name: &str, conn: &mut MysqlConnection) -> Result<u64, diesel::result::Error> {
    use roles::dsl as r;
    // Try to find
    if let Ok(found) = r::roles
        .filter(r::name.eq(name))
        .select(r::id)
        .first::<u64>(conn)
    {
        return Ok(found);
    }
    diesel::insert_into(r::roles)
        .values((r::name.eq(name),))
        .execute(conn)?;
    r::roles
        .filter(r::name.eq(name))
        .select(r::id)
        .first::<u64>(conn)
}

fn assign_role(
    user_id_v: u64,
    role_id_v: u64,
    conn: &mut MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use user_roles::dsl as ur;
    // Check if exists
    let exists: Result<(u64, u64), _> = ur::user_roles
        .filter(ur::user_id.eq(user_id_v).and(ur::role_id.eq(role_id_v)))
        .select((ur::user_id, ur::role_id))
        .first(conn);
    if exists.is_ok() {
        return Ok(());
    }
    let values = (ur::user_id.eq(user_id_v), ur::role_id.eq(role_id_v));
    let _ = diesel::insert_into(ur::user_roles)
        .values(values)
        .execute(conn)?;
    Ok(())
}

pub async fn register(
    pool: web::Data<DbPool>,
    payload: web::Json<RegisterRequest>,
) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Hash password
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => return e,
    };

    let new_user = NewUser {
        email: payload.email.clone(),
        name: payload.name.clone(),
        password_hash,
    };

    // Insert user
    if let Err(e) = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
    {
        eprintln!("Failed to create user: {}", e);
        return HttpResponse::BadRequest().body("user_create_failed");
    }

    // Fetch inserted user id
    let created: User = match users::table
        .filter(users::email.eq(&payload.email))
        .select(User::as_select())
        .first(&mut conn)
    {
        Ok(u) => u,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Assign roles: first user -> Admin, otherwise Homeowner by default
    let total = count_users(&mut conn);
    let role_name = if total == 1 { "Admin" } else { "Homeowner" };
    if let Ok(role_id) = ensure_role(role_name, &mut conn) {
        let _ = assign_role(created.id, role_id, &mut conn);
    }

    HttpResponse::Created().finish()
}

pub async fn login(
    pool: web::Data<DbPool>,
    keys: web::Data<JwtKeys>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    use users::dsl as u;
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let user: User = match u::users
        .filter(u::email.eq(&payload.email))
        .select(User::as_select())
        .first(&mut conn)
    {
        Ok(u) => u,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    if !verify_password(&payload.password, &user.password_hash) {
        return HttpResponse::Unauthorized().finish();
    }

    let roles = get_user_roles(user.id, &mut conn);

    let exp = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles,
        exp: exp.timestamp() as usize,
    };

    let token = match encode(&Header::new(Algorithm::HS256), &claims, &keys.encoding) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(AuthResponse { token })
}

pub fn extract_claims_from_req(req: &HttpRequest, keys: &JwtKeys) -> Option<Claims> {
    let auth = req
        .headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .to_string();
    let token = auth.strip_prefix("Bearer ")?.to_string();
    let data = decode::<Claims>(&token, &keys.decoding, &Validation::new(Algorithm::HS256)).ok()?;
    Some(data.claims)
}

pub fn has_any_role(claims: &Claims, wanted: &[&str]) -> bool {
    if wanted.is_empty() {
        return true;
    }
    claims.roles.iter().any(|r| wanted.iter().any(|w| r == w))
}
