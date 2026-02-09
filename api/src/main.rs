use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use api::i18n::{get_message, init_translations, negotiate_language};
use api::{
    AppConfig, DbPool, JwtKeys, MIGRATIONS, announcements, apartments, auth, buildings, dashboard,
    invitations, maintenance, meters, openapi::ApiDoc, users, voting,
};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::MigrationHarness;
use serde::Serialize;
use std::env;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Build CORS middleware from `CORS_ALLOWED_ORIGINS` env var.
///
/// - If the env var is set to `"*"`, allows any origin (development only).
/// - If the env var contains a comma-separated list of origins, only those are allowed.
/// - If the env var is unset, defaults to `http://localhost:8081` (local Trunk dev server).
fn build_cors() -> Cors {
    let allowed = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:8081,http://127.0.0.1:8081".to_string());

    if allowed.trim() == "*" {
        Cors::permissive()
    } else {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::ACCEPT_LANGUAGE,
            ])
            .max_age(3600);

        for origin in allowed.split(',') {
            let origin = origin.trim();
            if !origin.is_empty() {
                cors = cors.allowed_origin(origin);
            }
        }

        cors
    }
}

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

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,actix_web=info")),
        )
        .init();

    let port = env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting server at http://{}", addr);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = DbPool::builder()
        .max_size(
            env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        )
        .min_idle(Some(2))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool.");

    let pool_state = pool.state();
    info!(
        connections = pool_state.connections,
        idle = pool_state.idle_connections,
        "DB pool initialized"
    );

    {
        // Run migrations
        let mut conn = pool
            .get()
            .expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    init_translations();

    let jwt_secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set. Generate one with: openssl rand -base64 32");
    let keys = JwtKeys::from_secret(&jwt_secret);
    let app_config = AppConfig::load();
    info!(
        attachments_path = %app_config.attachments_base_path,
        max_size = app_config.max_attachment_size_bytes,
        mime_types = ?app_config.allowed_mime_types,
        "AppConfig loaded"
    );

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(keys.clone()))
            .app_data(web::Data::new(app_config.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api/v1")
                    .wrap(build_cors())
                    .route("/health", web::get().to(health))
                    .configure(auth::configure)
                    .configure(users::configure)
                    .configure(buildings::configure)
                    .configure(apartments::configure)
                    .configure(maintenance::configure)
                    .configure(announcements::configure)
                    .configure(voting::configure)
                    .configure(meters::configure)
                    .configure(dashboard::configure)
                    .configure(invitations::configure),
            )
    })
    .bind(addr)?
    .run()
    .await
}
