use crate::models::{InvitationStatus, PublicUser};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub(super) type RenterRow = (
    u64,
    u64,
    u64,
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
    Option<bool>,
    Option<chrono::NaiveDateTime>,
);

pub(super) type PropertyHistoryRow = (
    u64,
    u64,
    String,
    Option<u64>,
    u64,
    String,
    Option<String>,
    Option<chrono::NaiveDateTime>,
);

pub(super) type InvitationRow = (
    u64,
    u64,
    String,
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
    u64,
    InvitationStatus,
    chrono::NaiveDateTime,
    Option<chrono::NaiveDateTime>,
);

#[derive(Deserialize, ToSchema)]
pub struct OwnerAssignPayload {
    pub user_id: u64,
}

#[derive(Deserialize, ToSchema)]
pub struct RenterAssignPayload {
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct RenterUpdatePayload {
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct RenterWithUser {
    pub id: u64,
    pub apartment_id: u64,
    pub user_id: u64,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub user: PublicUser,
}

/// Apartment with building info for forms
#[derive(Serialize, ToSchema)]
pub struct ApartmentWithBuilding {
    pub id: u64,
    pub number: String,
    pub building_id: u64,
    pub building_address: String,
}

/// Apartment detail with building info
#[derive(Serialize, ToSchema)]
pub struct ApartmentDetail {
    pub id: u64,
    pub number: String,
    pub building_id: u64,
    pub building_address: String,
    pub size_sq_m: Option<f64>,
}

/// Apartment permissions for current user
#[derive(Serialize, ToSchema)]
pub struct ApartmentPermissions {
    pub can_view: bool,
    pub can_manage_renters: bool,
    pub can_view_meters: bool,
    pub is_owner: bool,
    pub is_renter: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct InviteRenterPayload {
    pub email: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

#[derive(Serialize, ToSchema)]
pub struct InviteRenterResponse {
    pub invitation_id: u64,
    pub email: String,
    pub status: String,
    pub message: String,
}
