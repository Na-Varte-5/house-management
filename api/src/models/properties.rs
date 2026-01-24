use crate::schema::{
    apartment_owners, apartment_renters, apartments, building_managers, buildings, property_history,
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
