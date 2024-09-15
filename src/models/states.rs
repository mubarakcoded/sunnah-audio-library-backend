#[derive(sqlx::FromRow, serde::Serialize)]
pub struct State {
    pub id: i32,
    pub name: String,
}
