use crate::schema::{
    apartment_owners, apartment_renters, apartments, building_managers, buildings,
    property_history, renter_invitations,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::users::User;

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

// Property History models
#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = property_history)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PropertyHistory {
    pub id: u64,
    pub apartment_id: u64,
    pub event_type: String,
    pub user_id: Option<u64>,
    pub changed_by: u64,
    pub description: String,
    pub metadata: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = property_history)]
pub struct NewPropertyHistory {
    pub apartment_id: u64,
    pub event_type: String,
    pub user_id: Option<u64>,
    pub changed_by: u64,
    pub description: String,
    pub metadata: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PropertyHistoryEnriched {
    pub id: u64,
    pub apartment_id: u64,
    pub event_type: String,
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub changed_by: u64,
    pub changed_by_name: String,
    pub description: String,
    pub metadata: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, diesel::AsExpression, diesel::FromSqlRow, ToSchema)]
#[diesel(sql_type = crate::schema::sql_types::RenterInvitationsStatusEnum)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Cancelled,
}

impl
    diesel::serialize::ToSql<
        crate::schema::sql_types::RenterInvitationsStatusEnum,
        diesel::mysql::Mysql,
    > for InvitationStatus
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::mysql::Mysql>,
    ) -> diesel::serialize::Result {
        use std::io::Write;
        match *self {
            InvitationStatus::Pending => out.write_all(b"pending")?,
            InvitationStatus::Accepted => out.write_all(b"accepted")?,
            InvitationStatus::Expired => out.write_all(b"expired")?,
            InvitationStatus::Cancelled => out.write_all(b"cancelled")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

impl
    diesel::deserialize::FromSql<
        crate::schema::sql_types::RenterInvitationsStatusEnum,
        diesel::mysql::Mysql,
    > for InvitationStatus
{
    fn from_sql(bytes: diesel::mysql::MysqlValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = <String as diesel::deserialize::FromSql<
            diesel::sql_types::Text,
            diesel::mysql::Mysql,
        >>::from_sql(bytes)?;
        match s.as_str() {
            "pending" => Ok(InvitationStatus::Pending),
            "accepted" => Ok(InvitationStatus::Accepted),
            "expired" => Ok(InvitationStatus::Expired),
            "cancelled" => Ok(InvitationStatus::Cancelled),
            _ => Err(format!("Unknown status: {}", s).into()),
        }
    }
}

impl serde::Serialize for InvitationStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InvitationStatus::Pending => serializer.serialize_str("pending"),
            InvitationStatus::Accepted => serializer.serialize_str("accepted"),
            InvitationStatus::Expired => serializer.serialize_str("expired"),
            InvitationStatus::Cancelled => serializer.serialize_str("cancelled"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for InvitationStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "pending" => Ok(InvitationStatus::Pending),
            "accepted" => Ok(InvitationStatus::Accepted),
            "expired" => Ok(InvitationStatus::Expired),
            "cancelled" => Ok(InvitationStatus::Cancelled),
            _ => Err(serde::de::Error::custom(format!("Unknown status: {}", s))),
        }
    }
}

#[derive(Queryable, Selectable, Serialize, Debug, ToSchema)]
#[diesel(table_name = renter_invitations)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct RenterInvitation {
    pub id: u64,
    pub apartment_id: u64,
    pub email: String,
    pub token: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub invited_by: u64,
    pub status: InvitationStatus,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub accepted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = renter_invitations)]
pub struct NewRenterInvitation {
    pub apartment_id: u64,
    pub email: String,
    pub token: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub invited_by: u64,
    pub status: InvitationStatus,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct RenterInvitationWithDetails {
    pub id: u64,
    pub apartment_id: u64,
    pub apartment_number: String,
    pub building_address: String,
    pub email: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub invited_by: u64,
    pub invited_by_name: String,
    pub status: InvitationStatus,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
}
