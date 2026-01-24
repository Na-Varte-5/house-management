// Domain-specific model modules
pub mod announcements;
pub mod maintenance;
pub mod meters;
pub mod properties;
pub mod users;
pub mod voting;

// Re-export all types for convenient importing
pub use announcements::*;
pub use maintenance::*;
pub use meters::*;
pub use properties::*;
pub use users::*;
pub use voting::*;
