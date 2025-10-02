use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::{users, roles, user_roles, buildings, apartments};

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Debug, Serialize)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: u64,
    pub name: String,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = user_roles)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
}

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = buildings)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Building {
    pub id: u64,
    pub address: String,
    pub construction_year: Option<i32>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = buildings)]
pub struct NewBuilding {
    pub address: String,
    pub construction_year: Option<i32>,
}

#[derive(Queryable, Selectable, Serialize, Debug)]
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
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = apartments)]
pub struct NewApartment {
    pub building_id: u64,
    pub number: String,
    pub size_sq_m: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
}
