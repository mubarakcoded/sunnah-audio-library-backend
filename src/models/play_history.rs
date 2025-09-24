use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayHistory {
    pub id: i32,
    pub user_id: i32,
    pub file_id: i32,
    pub played_duration: i32,
    pub device_type: Option<String>,
    pub played_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RecordPlayRequest {
    pub file_id: i32,
    pub played_duration: i32,
    pub device_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlayHistoryResponse {
    pub file_id: i32,
    pub file_title: String,
    pub scholar_name: Option<String>,
    pub played_duration: i32,
    pub device_type: Option<String>,
    pub played_at: NaiveDateTime,
}