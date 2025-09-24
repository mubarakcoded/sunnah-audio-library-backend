use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserScholarFollow {
    pub id: i32,
    pub user_id: i32,
    pub scholar_id: i32,
    pub notifications_enabled: bool,
    pub followed_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct FollowScholarRequest {
    pub scholar_id: i32,
    pub notifications_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFollowRequest {
    pub notifications_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct FollowResponse {
    pub scholar_id: i32,
    pub scholar_name: String,
    pub notifications_enabled: bool,
    pub followed_at: NaiveDateTime,
}