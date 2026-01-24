use crate::schema::users;
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
#[diesel(table_name = crate::schema::roles)]
#[allow(dead_code)]
pub struct Role {
    pub id: u64,
    pub name: String,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = crate::schema::user_roles)]
#[allow(dead_code)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
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
