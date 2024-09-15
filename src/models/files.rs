use chrono::{DateTime, Local};

#[derive(sqlx::FromRow, serde::Serialize)]
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

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RecentFiles {
    pub id: i32,
    pub file_name: String,
    pub scholar_name: String,
    pub scholar_image: String,
    pub date: DateTime<Local>,
}
