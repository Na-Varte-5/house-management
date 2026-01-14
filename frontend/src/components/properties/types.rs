use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, PartialEq)]
pub struct Building {
    pub id: u64,
    pub address: String,
    pub construction_year: Option<i32>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Apartment {
    pub id: u64,
    pub building_id: u64,
    pub number: String,
    pub size_sq_m: Option<f64>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct NewBuilding {
    pub address: String,
    pub construction_year: Option<i32>,
}

#[derive(Serialize)]
pub struct NewApartment {
    pub building_id: u64,
    pub number: String,
    pub size_sq_m: Option<f64>,
}

#[derive(Serialize)]
pub struct AssignOwnerRequest {
    pub user_id: u64,
}
