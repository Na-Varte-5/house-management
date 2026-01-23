use crate::schema::{
    apartment_owners, apartment_renters, apartments, building_managers, buildings,
    maintenance_request_attachments, maintenance_request_history, maintenance_requests,
    meter_readings, meters, proposal_results, proposals, users, votes, webhook_api_keys,
};
use bigdecimal::BigDecimal; // for voting weights
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Debug, Serialize)]
#[diesel(table_name = roles)]
#[allow(dead_code)]
pub struct Role {
    pub id: u64,
    pub name: String,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = user_roles)]
#[allow(dead_code)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = buildings)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Building {
    pub id: u64,
    pub address: String,
    pub construction_year: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub is_deleted: bool,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = buildings)]
pub struct NewBuilding {
    pub address: String,
    pub construction_year: Option<i32>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = apartments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Apartment {
    pub id: u64,
    pub building_id: u64,
    pub number: String,
    pub size_sq_m: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub is_deleted: bool,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = apartments)]
pub struct NewApartment {
    pub building_id: u64,
    pub number: String,
    pub size_sq_m: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
}

#[derive(Queryable, Insertable, Associations, Identifiable, Debug)]
#[diesel(table_name = apartment_owners)]
#[diesel(primary_key(apartment_id, user_id))]
#[diesel(belongs_to(Apartment, foreign_key = apartment_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct ApartmentOwner {
    pub apartment_id: u64,
    pub user_id: u64,
}

#[derive(Queryable, Insertable, Serialize, Associations, Identifiable, Debug, ToSchema)]
#[diesel(table_name = building_managers)]
#[diesel(primary_key(building_id, user_id))]
#[diesel(belongs_to(Building, foreign_key = building_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct BuildingManager {
    pub building_id: u64,
    pub user_id: u64,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = apartment_renters)]
#[diesel(belongs_to(Apartment, foreign_key = apartment_id))]
#[diesel(belongs_to(User, foreign_key = user_id))]
pub struct ApartmentRenter {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = apartment_renters)]
pub struct NewApartmentRenter {
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = maintenance_requests)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct MaintenanceRequest {
    pub id: u64,
    pub apartment_id: u64,
    pub created_by: u64,
    pub assigned_to: Option<u64>,
    pub request_type: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub resolution_notes: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = maintenance_requests)]
pub struct NewMaintenanceRequest {
    pub apartment_id: u64,
    pub request_type: String,
    pub priority: String,
    pub title: String,
    pub description: String,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = maintenance_request_attachments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct MaintenanceRequestAttachment {
    pub id: u64,
    pub request_id: u64,
    pub original_filename: String,
    pub stored_filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub is_deleted: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = maintenance_request_history)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct MaintenanceRequestHistory {
    pub id: u64,
    pub request_id: u64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub changed_by: u64,
    pub changed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = proposals)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub created_by: u64,
    pub building_id: Option<u64>,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub voting_method: String,
    pub eligible_roles: String,
    pub status: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = proposals)]
pub struct NewProposal {
    pub title: String,
    pub description: String,
    pub building_id: Option<u64>,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub voting_method: String,
    pub eligible_roles: String,
    pub status: String,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = votes)]
pub struct Vote {
    pub id: u64,
    pub proposal_id: u64,
    pub user_id: u64,
    #[schema(value_type = String, example = "1.5")]
    pub weight_decimal: BigDecimal,
    pub choice: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = proposal_results)]
pub struct ProposalResult {
    pub id: u64,
    pub proposal_id: u64,
    pub passed: bool,
    #[schema(value_type = String, example = "5.0")]
    pub yes_weight: BigDecimal,
    #[schema(value_type = String, example = "3.0")]
    pub no_weight: BigDecimal,
    #[schema(value_type = String, example = "1.0")]
    pub abstain_weight: BigDecimal,
    #[schema(value_type = String, example = "9.0")]
    pub total_weight: BigDecimal,
    pub tallied_at: Option<chrono::NaiveDateTime>,
    pub method_applied_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VotingMethod {
    SimpleMajority,
    WeightedArea,
    PerSeat,
    Consensus,
}
impl std::fmt::Display for VotingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SimpleMajority => "SimpleMajority",
                Self::WeightedArea => "WeightedArea",
                Self::PerSeat => "PerSeat",
                Self::Consensus => "Consensus",
            }
        )
    }
}
impl std::str::FromStr for VotingMethod {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "SimpleMajority" => Self::SimpleMajority,
            "WeightedArea" => Self::WeightedArea,
            "PerSeat" => Self::PerSeat,
            "Consensus" => Self::Consensus,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}
impl std::fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Yes => "Yes",
                Self::No => "No",
                Self::Abstain => "Abstain",
            }
        )
    }
}
impl std::str::FromStr for VoteChoice {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Yes" => Self::Yes,
            "No" => Self::No,
            "Abstain" => Self::Abstain,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ProposalStatus {
    Scheduled,
    Open,
    Closed,
    Tallied,
}
impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Scheduled => "Scheduled",
                Self::Open => "Open",
                Self::Closed => "Closed",
                Self::Tallied => "Tallied",
            }
        )
    }
}
impl std::str::FromStr for ProposalStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Scheduled" => Self::Scheduled,
            "Open" => Self::Open,
            "Closed" => Self::Closed,
            "Tallied" => Self::Tallied,
            _ => return Err(()),
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct PublicUser {
    pub id: u64,
    pub email: String,
    pub name: String,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        PublicUser {
            id: u.id,
            email: u.email,
            name: u.name,
        }
    }
}

use crate::schema::{announcements, announcements_comments};

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = announcements)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Announcement {
    pub id: u64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub author_id: u64,
    pub public: bool,
    pub pinned: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub apartment_id: Option<u64>,
    pub comments_enabled: bool,
    pub publish_at: Option<chrono::NaiveDateTime>,
    pub expire_at: Option<chrono::NaiveDateTime>,
    pub is_deleted: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = announcements)]
pub struct NewAnnouncement {
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub author_id: u64,
    pub public: bool,
    pub pinned: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub apartment_id: Option<u64>,
    pub comments_enabled: bool,
    pub publish_at: Option<chrono::NaiveDateTime>,
    pub expire_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = announcements_comments)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct AnnouncementComment {
    pub id: u64,
    pub announcement_id: u64,
    pub user_id: u64,
    pub body_md: String,
    pub body_html: String,
    pub is_deleted: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = announcements_comments)]
pub struct NewAnnouncementComment {
    pub announcement_id: u64,
    pub user_id: u64,
    pub body_md: String,
    pub body_html: String,
}

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
