use serde::Serialize;
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(FromRow, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct BookSearchResult {
    pub id: i32,
    pub name: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookDetails {
    pub id: i32,
    pub name: String,
    pub about: Option<String>,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub statistics: BookStatistics,
}

#[derive(Debug, Serialize)]
pub struct BookStatistics {
    pub total_files: i64,
    pub total_downloads: i64,
    pub total_plays: i64,
    pub total_likes: i64,
    pub average_rating: Option<f64>,
}
