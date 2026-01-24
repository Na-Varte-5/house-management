use crate::schema::{
    maintenance_request_attachments, maintenance_request_history, maintenance_requests,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
