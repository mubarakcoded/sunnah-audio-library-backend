use serde::Serialize;
use sqlx::FromRow;

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
