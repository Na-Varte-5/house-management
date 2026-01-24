use crate::schema::{meter_readings, meters, webhook_api_keys};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Meter models
#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = meters)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Meter {
    pub id: u64,
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    pub is_visible_to_renters: bool,
    pub installation_date: Option<chrono::NaiveDate>,
    pub calibration_due_date: Option<chrono::NaiveDate>,
    pub last_calibration_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = meters)]
pub struct NewMeter {
    pub apartment_id: u64,
    pub meter_type: String,
    pub serial_number: String,
    pub is_visible_to_renters: Option<bool>,
    pub installation_date: Option<chrono::NaiveDate>,
    pub calibration_due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum MeterType {
    ColdWater,
    HotWater,
    Gas,
    Electricity,
}

impl std::fmt::Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ColdWater => "ColdWater",
                Self::HotWater => "HotWater",
                Self::Gas => "Gas",
                Self::Electricity => "Electricity",
            }
        )
    }
}

impl std::str::FromStr for MeterType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ColdWater" => Self::ColdWater,
            "HotWater" => Self::HotWater,
            "Gas" => Self::Gas,
            "Electricity" => Self::Electricity,
            _ => return Err(()),
        })
    }
}

// Meter Reading models
#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = meter_readings)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct MeterReading {
    pub id: u64,
    pub meter_id: u64,
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub reading_timestamp: chrono::NaiveDateTime,
    pub unit: String,
    pub source: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = meter_readings)]
pub struct NewMeterReading {
    pub meter_id: u64,
    #[schema(value_type = String, example = "123.456")]
    pub reading_value: BigDecimal,
    pub reading_timestamp: chrono::NaiveDateTime,
    pub unit: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ReadingSource {
    Webhook,
    Manual,
}

impl std::fmt::Display for ReadingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Webhook => "Webhook",
                Self::Manual => "Manual",
            }
        )
    }
}

impl std::str::FromStr for ReadingSource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Webhook" => Self::Webhook,
            "Manual" => Self::Manual,
            _ => return Err(()),
        })
    }
}

// Webhook API Key models
#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = webhook_api_keys)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct WebhookApiKey {
    pub id: u64,
    pub name: String,
    pub api_key_hash: String,
    pub is_active: bool,
    pub created_by: u64,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub last_used_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = webhook_api_keys)]
pub struct NewWebhookApiKey {
    pub name: String,
    pub api_key_hash: String,
    pub created_by: u64,
}
