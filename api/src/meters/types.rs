use crate::models::Meter;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to create a new meter
#[derive(Deserialize, ToSchema)]
pub struct CreateMeterRequest {
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    #[serde(default = "default_visible_to_renters")]
    pub is_visible_to_renters: bool,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
}

fn default_visible_to_renters() -> bool {
    true
}

/// Request to update meter details
#[derive(Deserialize, ToSchema)]
pub struct UpdateMeterRequest {
    pub meter_type: Option<String>,
    pub serial_number: Option<String>,
    pub is_visible_to_renters: Option<bool>,
    pub installation_date: Option<String>,
    pub calibration_due_date: Option<String>,
}

/// Request to create a meter reading
#[derive(Deserialize, ToSchema)]
pub struct CreateReadingRequest {
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub timestamp: Option<String>,
    pub unit: String,
}

/// Request to calibrate a meter
#[derive(Deserialize, ToSchema)]
pub struct CalibrateMeterRequest {
    pub calibration_date: String,
    pub next_calibration_due: String,
}

/// Response type: meter with last reading information
#[derive(Serialize, ToSchema)]
pub struct MeterWithLastReading {
    #[serde(flatten)]
    pub meter: Meter,
    pub last_reading_value: Option<String>,
    pub last_reading_timestamp: Option<chrono::NaiveDateTime>,
    pub last_reading_unit: Option<String>,
}

/// Webhook payload for single meter reading
#[derive(Deserialize, ToSchema)]
pub struct WebhookReadingPayload {
    pub serial_number: String,
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub timestamp: String,
    pub unit: String,
}

/// Webhook payload for batch meter readings
#[derive(Deserialize, ToSchema)]
pub struct WebhookBatchPayload {
    pub readings: Vec<WebhookReadingPayload>,
}

/// Request to create a webhook API key
#[derive(Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

/// Response containing API key (only returned on creation)
#[derive(Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: u64,
    pub name: String,
    pub api_key: String,
}
