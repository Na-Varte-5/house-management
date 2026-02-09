pub mod announcements;
pub mod apartments;
pub mod auth;
pub mod buildings;
pub mod config;
pub mod dashboard;
pub mod db;
pub mod i18n;
pub mod invitations;
pub mod maintenance;
pub mod meters;
pub mod models;
pub mod openapi;
pub mod pagination;
pub mod schema;
pub mod users;
pub mod voting;

pub use auth::{
    JwtKeys,
    crypto::{hash_password, verify_password},
};
pub use config::AppConfig;
pub use db::DbPool;

use diesel_migrations::{EmbeddedMigrations, embed_migrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
