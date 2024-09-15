#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    // pub about: String,
    pub image: String,
    // pub created_by: i32,
}
