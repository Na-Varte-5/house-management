mod schema;
mod i18n;

use crate::schema::*;
use crate::i18n::{init_translations, negotiate_language, get_message};
use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Serialize, Deserialize};
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
    // Get the preferred language from the Accept-Language header
    let accept_language = req.headers().get("Accept-Language")
        .and_then(|header| header.to_str().ok());
    let lang = negotiate_language(accept_language);

    // Get translated message
    let message = get_message(&lang, "health-ok");

    web::Json(HealthResponse { 
        status: "ok".to_string(),
        message
    })
}

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: u64,
    pub name: String,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = user_roles)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
}

#[derive(Deserialize)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUserDB<'a> {
    pub email: &'a str,
    pub name: &'a str,
    pub password_hash: &'a str,
}

async fn list_users(pool: web::Data<DbPool>) -> impl Responder {
    use crate::schema::users::dsl::*;
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let result = users.select(User::as_select()).load(&mut conn);
    match result {
        Ok(user_list) => HttpResponse::Ok().json(user_list),
        Err(e) => {
            eprintln!("Error loading users: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

async fn create_user(pool: web::Data<DbPool>, item: web::Json<NewUser>) -> impl Responder {
    use crate::schema::users;
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let new_user = NewUserDB {
        email: &item.email,
        name: &item.name,
        password_hash: &item.password_hash,
    };
    let inserted = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn);
    match inserted {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
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
    println!("Initializing translations...");
    init_translations();
    println!("Translations initialized successfully.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
            .route("/users", web::get().to(list_users))
            .route("/users", web::post().to(create_user))
    })
    .bind(addr)?
    .run()
    .await
}
