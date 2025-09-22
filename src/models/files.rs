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
