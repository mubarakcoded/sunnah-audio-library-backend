use chrono::{DateTime, Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Files {
    pub id: i32,
    pub name: String,
    pub url: String,
    // pub uid: String,
    // pub r#type: String,  // "type" is a reserved keyword in Rust, using r#type
    pub size: String,
    pub duration: String,
    // pub scholar_id: i32,
    // pub book_id: i32,
    pub date: DateTime<Local>,
    pub downloads: i32,
    // pub status: String,
    // pub created_by: i32,
    // pub created_at: i64,
    // pub updated_at: i64,
}

#[derive(FromRow, Serialize)]
pub struct RecentFiles {
    pub id: i32,
    pub file_name: String,
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
