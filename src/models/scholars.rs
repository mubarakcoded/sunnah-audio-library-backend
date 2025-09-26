use serde::Serialize;
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(FromRow, Serialize)]
pub struct Scholar {
    pub id: i32,
    pub name: String,
    pub state: String,
    pub image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScholarSearchResult {
    pub id: i32,
    pub name: String,
    pub image: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScholarDetails {
    pub id: i32,
    pub name: String,
    pub about: Option<String>,
    pub state: String,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub statistics: ScholarStatistics,
}

#[derive(Debug, Serialize)]
pub struct ScholarStatistics {
    pub total_books: i64,
    pub total_files: i64,
    pub total_downloads: i64,
    pub total_plays: i64,
    pub total_likes: i64,
    pub total_followers: i64,
}
