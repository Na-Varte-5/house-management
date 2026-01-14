// Refactored auth module into internal submodules for handlers, helpers, extractor, and error types.
// NOTE: This is kept in a single file due to tooling constraints; can be split into a directory structure later.

pub mod building_access;
pub mod crypto;
pub mod error;
pub mod extractor;
pub mod handlers;
pub mod roles;
pub mod types;

pub use building_access::get_user_building_ids;
pub use error::AppError;
pub use extractor::AuthContext;
pub use handlers::configure;
pub use types::JwtKeys;
