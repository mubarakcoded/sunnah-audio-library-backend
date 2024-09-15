use crate::core::AppError;
use crate::models::scholars::Scholar;
use crate::models::states::State;
use sqlx::MySqlPool;

pub async fn fetch_scholars(pool: &MySqlPool) -> Result<Vec<Scholar>, AppError> {
    let scholars = sqlx::query_as!(Scholar, "SELECT id, name, about AS state, image FROM tbl_scholars")
        .fetch_all(pool)
        .await
        .map_err(AppError::db_error)?;

    Ok(scholars)
}

pub async fn fetch_scholars_by_state(
    pool: &MySqlPool,
    state_id: i32,
) -> Result<Vec<Scholar>, AppError> {
    let scholars = sqlx::query_as!(
        Scholar,
        "SELECT id, name, about AS state, image FROM tbl_scholars WHERE state = ?",
        state_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(scholars)
}
