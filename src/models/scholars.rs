#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Scholar {
    pub id: i32,
    pub name: String,
    // pub state: i32,
    pub state: String,
    pub image: String,
    // pub status: String,
}
