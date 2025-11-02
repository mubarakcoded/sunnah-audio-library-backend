use serde::{Serialize, Deserialize};
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
    pub state_id: i32,
    pub state: String,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: i32,
    pub statistics: ScholarStatistics,
    pub is_followed_by_user: Option<bool>, // Will be None if no user context
    pub has_access: Option<bool>, // Will be None if no user context, true if manager has access
}

#[derive(Debug, Serialize)]
pub struct ScholarDropdown {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateScholarRequest {
    pub name: String,
    pub about: Option<String>,
    pub state_id: i32,
    pub image: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateScholarRequest {
    pub name: Option<String>,
    pub about: Option<String>,
    pub state_id: Option<i32>,
    pub image: Option<String>,
    pub priority: Option<i32>,
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
