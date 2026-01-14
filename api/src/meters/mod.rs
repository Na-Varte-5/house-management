// Meters module - organized into logical sub-modules
//
// This module handles all meter-related functionality including:
// - Meter registration and management (handlers.rs)
// - Meter readings and exports (readings.rs)
// - Calibration tracking (calibration.rs)
// - Webhook integration for automated data ingestion (webhooks.rs)
// - API key management for webhook authentication (api_keys.rs)

mod types;
mod helpers;
mod handlers;
mod readings;
mod calibration;
mod webhooks;
mod api_keys;

// Re-export types for use in other modules
pub use types::*;

// Re-export handler functions
use actix_web::web;

pub use handlers::{
    list_apartment_meters,
    get_meter,
    create_meter,
    update_meter,
    deactivate_meter,
};

pub use readings::{
    list_readings,
    create_reading,
    export_readings_csv,
};

pub use calibration::{
    list_calibration_due,
    calibrate_meter,
};

pub use webhooks::{
    webhook_meter_reading,
    webhook_meter_reading_batch,
};

pub use api_keys::{
    list_api_keys,
    create_api_key,
    revoke_api_key,
};

/// Configure routes for the meters module
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Meter CRUD operations
        .route("/apartments/{apartment_id}/meters", web::get().to(list_apartment_meters))
        .route("/meters", web::post().to(create_meter))
        // Specific routes must come before generic {id} routes
        .route("/meters/calibration-due", web::get().to(list_calibration_due))
        .route("/meters/{id}", web::get().to(get_meter))
        .route("/meters/{id}", web::put().to(update_meter))
        .route("/meters/{id}", web::delete().to(deactivate_meter))
        // Readings sub-routes
        .route("/meters/{id}/readings/export", web::get().to(export_readings_csv))
        .route("/meters/{id}/readings", web::get().to(list_readings))
        .route("/meters/{id}/readings", web::post().to(create_reading))
        // Calibration
        .route("/meters/{id}/calibrate", web::post().to(calibrate_meter))
        // Webhooks (no authentication required, uses API key)
        .route("/webhooks/meter-reading", web::post().to(webhook_meter_reading))
        .route("/webhooks/meter-reading-batch", web::post().to(webhook_meter_reading_batch))
        // API key management (Admin only)
        .route("/admin/api-keys", web::get().to(list_api_keys))
        .route("/admin/api-keys", web::post().to(create_api_key))
        .route("/admin/api-keys/{id}", web::delete().to(revoke_api_key));
}
