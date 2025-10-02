mod auth;
mod i18n;
mod models;
mod schema;

use crate::auth::{JwtKeys, extract_claims_from_req, has_any_role};
use crate::i18n::{get_message, init_translations, negotiate_language};
use crate::models::{
    Apartment, ApartmentOwner, Building, NewApartment, NewBuilding, NewUser, PublicUser, User,
};
use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde::Serialize;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

async fn health(req: HttpRequest) -> impl Responder {
    let accept_language = req
        .headers()
        .get("Accept-Language")
        .and_then(|h| h.to_str().ok());
    let lang = negotiate_language(accept_language);
    let message = get_message(&lang, "health-ok");

    web::Json(HealthResponse {
        status: "ok".into(),
        message,
    })
}

async fn list_users(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match users.select(User::as_select()).load(&mut conn) {
        Ok(user_list) => HttpResponse::Ok().json(user_list),
        Err(e) => {
            eprintln!("Error loading users: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn create_user(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    item: web::Json<NewUser>,
    keys: web::Data<JwtKeys>,
) -> impl Responder {
    use crate::schema::users::dsl as users_dsl;
    // AuthZ: Admin only
    if let Some(claims) = extract_claims_from_req(&req, &keys) {
        if !has_any_role(&claims, &["Admin"]) {
            return HttpResponse::Forbidden().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    }

    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match diesel::insert_into(users_dsl::users)
        .values(&*item)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("Failed to insert user: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Buildings handlers
async fn list_buildings(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::buildings::dsl::*;
    let mut conn = pool.get().expect("db pool");
    match buildings.select(Building::as_select()).load(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            eprintln!("Error loading buildings: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn create_building(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    item: web::Json<NewBuilding>,
    keys: web::Data<JwtKeys>,
) -> impl Responder {
    use crate::schema::buildings::dsl as b_dsl;
    // AuthZ: Admin or Manager
    if let Some(claims) = extract_claims_from_req(&req, &keys) {
        if !has_any_role(&claims, &["Admin", "Manager"]) {
            return HttpResponse::Forbidden().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    }

    let mut conn = pool.get().expect("db pool");
    match diesel::insert_into(b_dsl::buildings)
        .values(&*item)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("Failed to insert building: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Apartments handlers
async fn list_apartments(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::apartments::dsl::*;
    let mut conn = pool.get().expect("db pool");
    match apartments.select(Apartment::as_select()).load(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            eprintln!("Error loading apartments: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn list_building_apartments(path: web::Path<u64>, pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::apartments::dsl::*;
    let building = path.into_inner();
    let mut conn = pool.get().expect("db pool");
    match apartments
        .filter(building_id.eq(building))
        .select(Apartment::as_select())
        .load(&mut conn)
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            eprintln!("Error loading apartments: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn create_apartment(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    item: web::Json<NewApartment>,
    keys: web::Data<JwtKeys>,
) -> impl Responder {
    use crate::schema::apartments::dsl as a_dsl;
    // AuthZ: Admin or Manager
    if let Some(claims) = extract_claims_from_req(&req, &keys) {
        if !has_any_role(&claims, &["Admin", "Manager"]) {
            return HttpResponse::Forbidden().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    }

    let mut conn = pool.get().expect("db pool");
    match diesel::insert_into(a_dsl::apartments)
        .values(&*item)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("Failed to insert apartment: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Apartment owners handlers
#[derive(serde::Deserialize)]
struct OwnerAssignPayload {
    user_id: u64,
}

async fn list_apartment_owners(path: web::Path<u64>, pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::users::dsl as u;
    let apartment = path.into_inner();
    let mut conn = pool.get().expect("db pool");
    let res = ao::apartment_owners
        .inner_join(u::users.on(u::id.eq(ao::user_id)))
        .filter(ao::apartment_id.eq(apartment))
        .select(User::as_select())
        .load::<User>(&mut conn);
    match res {
        Ok(list) => {
            let pub_users: Vec<PublicUser> = list.into_iter().map(PublicUser::from).collect();
            HttpResponse::Ok().json(pub_users)
        }
        Err(e) => {
            eprintln!("Error loading owners: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn add_apartment_owner(
    req: HttpRequest,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<OwnerAssignPayload>,
    keys: web::Data<JwtKeys>,
) -> impl Responder {
    use crate::schema::apartment_owners::dsl as ao;
    // AuthZ: Admin or Manager
    if let Some(claims) = extract_claims_from_req(&req, &keys) {
        if !has_any_role(&claims, &["Admin", "Manager"]) {
            return HttpResponse::Forbidden().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    }

    let apartment = path.into_inner();
    let new = ApartmentOwner {
        apartment_id: apartment,
        user_id: payload.user_id,
    };
    let mut conn = pool.get().expect("db pool");
    // Upsert-like: ignore if exists
    let exists: Result<(u64, u64), _> = ao::apartment_owners
        .filter(
            ao::apartment_id
                .eq(new.apartment_id)
                .and(ao::user_id.eq(new.user_id)),
        )
        .select((ao::apartment_id, ao::user_id))
        .first(&mut conn);
    if exists.is_ok() {
        return HttpResponse::NoContent().finish();
    }

    match diesel::insert_into(ao::apartment_owners)
        .values(&new)
        .execute(&mut conn)
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            eprintln!("Failed to assign owner: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn remove_apartment_owner(
    req: HttpRequest,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
    keys: web::Data<JwtKeys>,
) -> impl Responder {
    use crate::schema::apartment_owners::dsl as ao;
    // AuthZ: Admin or Manager
    if let Some(claims) = extract_claims_from_req(&req, &keys) {
        if !has_any_role(&claims, &["Admin", "Manager"]) {
            return HttpResponse::Forbidden().finish();
        }
    } else {
        return HttpResponse::Unauthorized().finish();
    }

    let (apartment, user) = path.into_inner();
    let mut conn = pool.get().expect("db pool");
    match diesel::delete(
        ao::apartment_owners.filter(ao::apartment_id.eq(apartment).and(ao::user_id.eq(user))),
    )
    .execute(&mut conn)
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            eprintln!("Failed to remove owner: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let port = env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server at http://{}", addr);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = DbPool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Run migrations
    {
        let mut conn = pool
            .get()
            .expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    // Initialize translations
    init_translations();

    // JWT keys
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let keys = JwtKeys::from_secret(&jwt_secret);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(keys.clone()))
            .service(
                web::scope("/api/v1")
                    .wrap(Cors::permissive())
                    .route("/health", web::get().to(health))
                    // Auth
                    .service(
                        web::resource("/auth/register")
                            .route(web::post().to(auth::register))
                            .route(
                                web::route()
                                    .method(actix_web::http::Method::OPTIONS)
                                    .to(|| async { HttpResponse::NoContent().finish() }),
                            ),
                    )
                    .service(
                        web::resource("/auth/login")
                            .route(web::post().to(auth::login))
                            .route(
                                web::route()
                                    .method(actix_web::http::Method::OPTIONS)
                                    .to(|| async { HttpResponse::NoContent().finish() }),
                            ),
                    )
                    // Users
                    .route("/users", web::get().to(list_users))
                    .route("/users", web::post().to(create_user))
                    // Buildings
                    .route("/buildings", web::get().to(list_buildings))
                    .route("/buildings", web::post().to(create_building))
                    // Apartments
                    .route("/apartments", web::get().to(list_apartments))
                    .route("/apartments", web::post().to(create_apartment))
                    .route(
                        "/buildings/{id}/apartments",
                        web::get().to(list_building_apartments),
                    )
                    // Apartment owners
                    .route(
                        "/apartments/{id}/owners",
                        web::get().to(list_apartment_owners),
                    )
                    .route(
                        "/apartments/{id}/owners",
                        web::post().to(add_apartment_owner),
                    )
                    .route(
                        "/apartments/{id}/owners/{user_id}",
                        web::delete().to(remove_apartment_owner),
                    ),
            )
    })
    .bind(addr)?
    .run()
    .await
}
