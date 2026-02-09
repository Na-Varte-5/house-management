use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub body_md: String,
    pub public: bool,
    pub pinned: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub apartment_id: Option<u64>,
    pub comments_enabled: bool,
    pub publish_at: Option<chrono::NaiveDateTime>,
    pub expire_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAnnouncementRequest {
    pub title: Option<String>,
    pub body_md: Option<String>,
    pub public: Option<bool>,
    pub pinned: Option<bool>,
    pub roles_csv: Option<Option<String>>, // double option to allow clearing
    pub building_id: Option<Option<u64>>,
    pub apartment_id: Option<Option<u64>>,
    pub comments_enabled: Option<bool>,
    pub publish_at: Option<Option<chrono::NaiveDateTime>>,
    pub expire_at: Option<Option<chrono::NaiveDateTime>>,
}

#[derive(Deserialize, IntoParams)]
pub struct CommentsQuery {
    pub include_deleted: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    pub body_md: String,
}

#[derive(Serialize, ToSchema)]
pub struct AnnouncementOut {
    pub id: u64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub author_id: u64,
    pub author_name: String,
    pub public: bool,
    pub pinned: bool,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub building_address: Option<String>,
    pub apartment_id: Option<u64>,
    pub apartment_number: Option<String>,
    pub comments_enabled: bool,
    pub publish_at: Option<chrono::NaiveDateTime>,
    pub expire_at: Option<chrono::NaiveDateTime>,
    pub is_deleted: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, ToSchema)]
pub struct CommentOut {
    pub id: u64,
    pub announcement_id: u64,
    pub user_id: u64,
    pub user_name: String,
    pub body_md: String,
    pub body_html: String,
    pub is_deleted: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}
