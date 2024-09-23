use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    // pub about: String,
    pub image: String,
    // pub created_by: i32,
}

#[derive(Debug, Serialize)]
pub struct BookSearchResult {
    pub id: i32,
    // pub scholar_id: i32,
    pub name: Option<String>,
    pub image: Option<String>,
}
