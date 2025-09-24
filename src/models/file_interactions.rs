use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

// File Reports
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReport {
    pub id: i32,
    pub user_id: i32,
    pub file_id: i32,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub admin_notes: Option<String>,
    pub resolved_by: Option<i32>,
    pub created_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub file_id: i32,
    pub reason: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveReportRequest {
    pub status: String, // reviewed, resolved, dismissed
    pub admin_notes: Option<String>,
}

// File Likes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileLike {
    pub id: i32,
    pub user_id: i32,
    pub file_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct LikeFileRequest {
    pub file_id: i32,
}

// File Comments
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileComment {
    pub id: i32,
    pub user_id: i32,
    pub file_id: i32,
    pub parent_id: Option<i32>,
    pub comment: String,
    pub is_approved: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub file_id: i32,
    pub parent_id: Option<i32>,
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub comment: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommentResponse {
    pub id: i32,
    pub user_name: String,
    pub parent_id: Option<i32>,
    pub comment: String,
    pub is_approved: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub replies: Vec<CommentResponse>,
}

// Download Logs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadLog {
    pub id: i32,
    pub user_id: i32,
    pub subscription_id: Option<i32>,
    pub file_id: i32,
    pub download_ip: Option<String>,
    pub user_agent: Option<String>,
    pub downloaded_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct DownloadStats {
    pub total_downloads: i64,
    pub unique_users: i64,
    pub downloads_today: i64,
    pub downloads_this_month: i64,
}