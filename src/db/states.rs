use sqlx::MySqlPool;
use crate::models::states::State;
use crate::core::AppError;

pub async fn fetch_states(pool: &MySqlPool) -> Result<Vec<State>, AppError> {
    let states = sqlx::query_as!(
        State,
        "SELECT id, name FROM tbl_states"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::db_error(e.to_string()))?;

    Ok(states)
}
