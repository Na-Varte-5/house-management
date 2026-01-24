use crate::schema::{announcements, announcements_comments};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
