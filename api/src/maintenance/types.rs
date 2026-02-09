use serde::Serialize;
use utoipa::ToSchema;

pub(super) type MaintenanceRequestQueryRow = (
    u64,                           // id
    u64,                           // apartment_id
    u64,                           // created_by
    Option<u64>,                   // assigned_to
    String,                        // request_type
    String,                        // priority
    String,                        // title
    String,                        // description
    String,                        // status
    Option<chrono::NaiveDateTime>, // created_at
    String,                        // apartment_number
    u64,                           // building_id
    String,                        // building_address
);

pub(super) type CommentRow = (
    u64,
    u64,
    u64,
    String,
    bool,
    Option<chrono::NaiveDateTime>,
    Option<chrono::NaiveDateTime>,
    String,
);

pub(super) type MaintenanceRequestDetailRow = (
    u64,                           // id
    u64,                           // apartment_id
    u64,                           // created_by
    Option<u64>,                   // assigned_to
    String,                        // request_type
    String,                        // priority
    String,                        // title
    String,                        // description
    String,                        // status
    Option<String>,                // resolution_notes
    Option<chrono::NaiveDateTime>, // created_at
    Option<chrono::NaiveDateTime>, // updated_at
    String,                        // apartment_number
    u64,                           // building_id
    String,                        // building_address
    String,                        // creator_name
);

#[derive(Serialize, ToSchema)]
pub struct MaintenanceRequestEnriched {
    pub id: u64,
    pub apartment_id: u64,
    pub apartment_number: String,
    pub building_id: u64,
    pub building_address: String,
    pub request_type: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_by: u64,
    pub assigned_to: Option<u64>,
    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct MaintenanceRequestDetail {
    pub id: u64,
    pub apartment_id: u64,
    pub apartment_number: String,
    pub building_id: u64,
    pub building_address: String,
    pub request_type: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub resolution_notes: Option<String>,
    pub created_by: u64,
    pub created_by_name: String,
    pub assigned_to: Option<u64>,
    pub assigned_to_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct MaintenanceRequestHistoryEnriched {
    pub id: u64,
    pub request_id: u64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub changed_by: u64,
    pub changed_by_name: String,
    pub changed_at: Option<String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct StatusUpdatePayload {
    #[schema(example = "InProgress")]
    pub status: String,
    pub note: Option<String>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct UpdateRequestPayload {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<u64>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct AssignPayload {
    pub user_id: u64,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct EscalatePayload {
    pub manager_id: u64,
}
