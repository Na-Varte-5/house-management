mod apartments;
mod auth;
mod buildings; // new
mod db; // new
mod i18n;
mod models;
mod schema;
mod users; // new // new

use crate::auth::JwtKeys;
use crate::db::DbPool;
use crate::i18n::{get_message, init_translations, negotiate_language};
use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use serde::Serialize;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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

    {
        // Run migrations
        let mut conn = pool
            .get()
            .expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    init_translations();

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
                    .configure(auth::configure)
                    .configure(users::configure)
                    .configure(buildings::configure)
                    .configure(apartments::configure),
            )
    })
    .bind(addr)?
    .run()
    .await
}
