mod schema;
mod i18n;
mod models;

use crate::schema::*;
use crate::i18n::{init_translations, negotiate_language, get_message};
use crate::models::{User, NewUser, NewBuilding, Building, NewApartment, Apartment};
use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::Serialize;
use std::env;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::prelude::*;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

async fn health(req: HttpRequest) -> impl Responder {
    let accept_language = req.headers().get("Accept-Language").and_then(|h| h.to_str().ok());
    let lang = negotiate_language(accept_language);
    let message = get_message(&lang, "health-ok");

    web::Json(HealthResponse { status: "ok".into(), message })
}

async fn list_users(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match users.select(User::as_select()).load(&mut conn) {
        Ok(user_list) => HttpResponse::Ok().json(user_list),
        Err(e) => { eprintln!("Error loading users: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

async fn create_user(pool: web::Data<DbPool>, item: web::Json<NewUser>) -> impl Responder {
    use crate::schema::users::dsl as users_dsl;
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match diesel::insert_into(users_dsl::users).values(&*item).execute(&mut conn) {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => { eprintln!("Failed to insert user: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

// Buildings handlers
async fn list_buildings(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::buildings::dsl::*;
    let mut conn = pool.get().expect("db pool");
    match buildings.select(Building::as_select()).load(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => { eprintln!("Error loading buildings: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

async fn create_building(pool: web::Data<DbPool>, item: web::Json<NewBuilding>) -> impl Responder {
    use crate::schema::buildings::dsl as b_dsl;
    let mut conn = pool.get().expect("db pool");
    match diesel::insert_into(b_dsl::buildings).values(&*item).execute(&mut conn) {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => { eprintln!("Failed to insert building: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

// Apartments handlers
async fn list_apartments(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::apartments::dsl::*;
    let mut conn = pool.get().expect("db pool");
    match apartments.select(Apartment::as_select()).load(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => { eprintln!("Error loading apartments: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

async fn list_building_apartments(path: web::Path<u64>, pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::apartments::dsl::*;
    let building = path.into_inner();
    let mut conn = pool.get().expect("db pool");
    match apartments.filter(building_id.eq(building)).select(Apartment::as_select()).load(&mut conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => { eprintln!("Error loading apartments: {}", e); HttpResponse::InternalServerError().finish() }
    }
}

async fn create_apartment(pool: web::Data<DbPool>, item: web::Json<NewApartment>) -> impl Responder {
    use crate::schema::apartments::dsl as a_dsl;
    let mut conn = pool.get().expect("db pool");
    match diesel::insert_into(a_dsl::apartments).values(&*item).execute(&mut conn) {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => { eprintln!("Failed to insert apartment: {}", e); HttpResponse::InternalServerError().finish() }
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
    let pool = DbPool::builder().build(manager).expect("Failed to create pool.");

    // Run migrations
    {
        let mut conn = pool.get().expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
    }

    // Initialize translations
    init_translations();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health))
                    .route("/users", web::get().to(list_users))
                    .route("/users", web::post().to(create_user))
                    .route("/buildings", web::get().to(list_buildings))
                    .route("/buildings", web::post().to(create_building))
                    .route("/apartments", web::get().to(list_apartments))
                    .route("/apartments", web::post().to(create_apartment))
                    .route("/buildings/{id}/apartments", web::get().to(list_building_apartments))
            )
    })
    .bind(addr)?
    .run()
    .await
}
