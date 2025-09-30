use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Files {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_size: String,
    pub book_id: i32,
    pub file_duration: String,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub scholar_image: String,
    pub date: DateTime<Local>,
    pub downloads: i32,
}

#[derive(Serialize)]
pub struct FilesWithStats {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_size: String,
    pub book_id: i32,
    pub file_duration: String,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub scholar_image: String,
    pub date: DateTime<Local>,
    pub statistics: FileStatistics,
}

#[derive(FromRow, Serialize)]
pub struct RecentFiles {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_size: String,
    pub file_duration: String,
    pub downloads: i32,
    pub book_id: i32,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub scholar_image: String,
    pub date: DateTime<Local>,
}

#[derive(Serialize)]
pub struct RecentFilesWithStats {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_size: String,
    pub file_duration: String,
    pub book_id: i32,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub scholar_image: String,
    pub date: DateTime<Local>,
    pub statistics: FileStatistics,
}

#[derive(Debug, Serialize)]
pub struct FileStatistics {
    pub total_downloads: i64,
    pub total_plays: i64,
    pub total_likes: i64,
    pub total_comments: i64,
    pub is_liked_by_user: Option<bool>, // Will be None if no user context
}

#[derive(Debug, Serialize)]
pub struct FileSearchResult {
    pub id: i32,
    pub file_name: String,
    pub scholar_name: String,
    pub image: Option<String>, // Scholar image URL
    pub date: DateTime<Local>,
}

#[derive(Debug, Serialize)]
pub struct ViewFileDetails {
    pub file_id: i32,
    pub file_name: String,
    pub duration: String,
    pub size: String,
    pub created_at: DateTime<Local>,
    pub book_image: Option<String>, // URL to the book's image
}

#[derive(Debug, Serialize)]
pub struct RelatedFiles {
    pub id: i32,
    pub name: String,
    pub duration: String,
    pub size: String,
    pub downloads: i32,
    pub date: DateTime<Local>,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct RelatedFilesParams {
    pub page: Option<i64>,
    pub items_per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PlayAllFile {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_size: String,
    pub file_duration: String,
    pub sort_order: Option<i32>, // For proper ordering in playlist
    pub date: DateTime<Local>,
}

#[derive(Debug, Serialize)]
pub struct PlayAllResponse {
    pub book_id: i32,
    pub book_name: String,
    pub book_image: Option<String>,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub scholar_image: Option<String>,
    pub total_files: i32,
    pub total_duration: Option<String>, // Sum of all file durations
    pub files: Vec<PlayAllFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFileRequest {
    pub name: Option<String>,
    pub book_id: Option<i32>,
    pub scholar_id: Option<i32>,
}
