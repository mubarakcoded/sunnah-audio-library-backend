use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Scholar {
    pub id: i32,
    pub name: String,
    // pub state: i32,
    pub state: String,
    pub image: Option<String>,
    // pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ScholarSearchResult {
    pub id: i32,
    pub name: String,
    pub image: Option<String>,
    pub state: Option<String>,
}
