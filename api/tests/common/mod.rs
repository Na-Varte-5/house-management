#![allow(dead_code)] // Test utilities may not be used in all test files

use actix_web::{App, HttpServer, web};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::MigrationHarness;
use serde_json::Value;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

/// Test database configuration
pub fn get_test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "mysql://home:home@localhost:3306/home_test".to_string())
}

/// Create a test database pool
pub fn create_test_pool() -> DbPool {
    let database_url = get_test_db_url();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    DbPool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create test pool")
}

/// Run migrations on test database
pub fn run_migrations(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(api::MIGRATIONS)
        .expect("Failed to run migrations");
}

/// Clean all tables in test database (for test isolation)
pub fn clean_database(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get connection");

    // Disable foreign key checks temporarily
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut conn)
        .expect("Failed to disable FK checks");

    // List of all tables to clean
    let tables = vec![
        "votes",
        "proposal_results",
        "proposals",
        "announcement_comments",
        "announcements",
        "maintenance_request_history",
        "maintenance_request_attachments",
        "maintenance_requests",
        "apartment_owners",
        "apartments",
        "buildings",
        "user_roles",
        "users",
        "roles",
    ];

    for table in tables {
        diesel::sql_query(format!("TRUNCATE TABLE {}", table))
            .execute(&mut conn)
            .ok(); // Ignore errors if table doesn't exist
    }

    // Re-enable foreign key checks
    diesel::sql_query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut conn)
        .expect("Failed to enable FK checks");
}

/// Test user data
#[derive(Clone)]
pub struct TestUser {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
    pub roles: Vec<String>,
    pub token: Option<String>,
}

impl TestUser {
    pub fn admin() -> Self {
        Self {
            id: 0,
            email: "admin@test.com".to_string(),
            name: "Test Admin".to_string(),
            password: "admin123".to_string(),
            roles: vec!["Admin".to_string()],
            token: None,
        }
    }

    pub fn manager() -> Self {
        Self {
            id: 0,
            email: "manager@test.com".to_string(),
            name: "Test Manager".to_string(),
            password: "manager123".to_string(),
            roles: vec!["Manager".to_string()],
            token: None,
        }
    }

    pub fn homeowner() -> Self {
        Self {
            id: 0,
            email: "homeowner@test.com".to_string(),
            name: "Test Homeowner".to_string(),
            password: "homeowner123".to_string(),
            roles: vec!["Homeowner".to_string()],
            token: None,
        }
    }

    pub fn renter() -> Self {
        Self {
            id: 0,
            email: "renter@test.com".to_string(),
            name: "Test Renter".to_string(),
            password: "renter123".to_string(),
            roles: vec!["Renter".to_string()],
            token: None,
        }
    }
}

/// Create a test user in the database and return the user with ID
pub async fn create_test_user(pool: &DbPool, mut user: TestUser) -> TestUser {
    use api::hash_password;
    use api::schema::{roles, user_roles, users};

    let mut conn = pool.get().expect("Failed to get connection");
    let hashed_password = hash_password(&user.password).expect("Failed to hash password");

    // Insert user
    diesel::insert_into(users::table)
        .values((
            users::email.eq(&user.email),
            users::name.eq(&user.name),
            users::password_hash.eq(&hashed_password),
        ))
        .execute(&mut conn)
        .expect("Failed to insert user");

    // Get user ID
    let user_id: u64 = diesel::select(diesel::dsl::sql::<
        diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
    >("LAST_INSERT_ID()"))
    .first(&mut conn)
    .expect("Failed to get user ID");

    user.id = user_id;

    // Insert roles and user_roles
    for role_name in &user.roles {
        // Get or create role
        let role_id: u64 = roles::table
            .filter(roles::name.eq(role_name))
            .select(roles::id)
            .first(&mut conn)
            .unwrap_or_else(|_| {
                diesel::insert_into(roles::table)
                    .values(roles::name.eq(role_name))
                    .execute(&mut conn)
                    .expect("Failed to insert role");

                diesel::select(diesel::dsl::sql::<
                    diesel::sql_types::Unsigned<diesel::sql_types::BigInt>,
                >("LAST_INSERT_ID()"))
                .first(&mut conn)
                .expect("Failed to get role ID")
            });

        // Assign role to user
        diesel::insert_into(user_roles::table)
            .values((
                user_roles::user_id.eq(user_id),
                user_roles::role_id.eq(role_id),
            ))
            .execute(&mut conn)
            .expect("Failed to assign role");
    }

    user
}

/// Login a test user and get JWT token
pub async fn login_test_user(client: &reqwest::Client, base_url: &str, user: &TestUser) -> String {
    let response = client
        .post(format!("{}/login", base_url))
        .json(&serde_json::json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .expect("Failed to login");

    assert!(
        response.status().is_success(),
        "Login failed for {}",
        user.email
    );

    let json: Value = response
        .json()
        .await
        .expect("Failed to parse login response");
    json["token"]
        .as_str()
        .expect("No token in response")
        .to_string()
}

/// Create a test user with token
pub async fn create_and_login_user(
    pool: &DbPool,
    client: &reqwest::Client,
    base_url: &str,
    user: TestUser,
) -> TestUser {
    let user = create_test_user(pool, user).await;
    let token = login_test_user(client, base_url, &user).await;
    TestUser {
        token: Some(token),
        ..user
    }
}

/// Test server configuration
pub struct TestServer {
    pub base_url: String,
    pub pool: DbPool,
}

impl TestServer {
    /// Start a test server on a random port
    pub async fn start() -> Self {
        dotenvy::dotenv().ok();

        let pool = create_test_pool();
        run_migrations(&pool);
        clean_database(&pool);

        // Start server on port 0 (random available port)
        let port = 8765; // Use fixed port for now, can make dynamic later
        let addr = format!("127.0.0.1:{}", port);
        let base_url = format!("http://{}/api/v1", addr);

        let pool_clone = pool.clone();
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
        let keys = api::auth::JwtKeys::from_secret(&jwt_secret);
        let app_config = api::config::AppConfig::load();

        // Start server in background
        tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(pool_clone.clone()))
                    .app_data(web::Data::new(keys.clone()))
                    .app_data(web::Data::new(app_config.clone()))
                    .service(
                        web::scope("/api/v1")
                            .configure(api::auth::configure)
                            .configure(api::users::configure)
                            .configure(api::buildings::configure)
                            .configure(api::apartments::configure)
                            .configure(api::maintenance::configure)
                            .configure(api::announcements::configure)
                            .configure(api::voting::configure),
                    )
            })
            .bind(&addr)
            .expect("Failed to bind test server")
            .run()
            .await
            .expect("Test server failed");
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Self { base_url, pool }
    }

    /// Clean the database between tests
    pub fn clean(&self) {
        clean_database(&self.pool);
    }
}
