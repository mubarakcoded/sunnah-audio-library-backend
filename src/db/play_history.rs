use crate::core::AppError;
use crate::models::play_history::{PlayHistory, PlayHistoryResponse, RecordPlayRequest};
use chrono::Utc;
use sqlx::MySqlPool;

// Record play history
pub async fn record_play(
    pool: &MySqlPool,
    user_id: i32,
    request: &RecordPlayRequest,
) -> Result<PlayHistory, AppError> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_play_history (
            user_id, 
            file_id, 
            played_duration, 
            total_duration, 
            play_position, 
            play_action, 
            device_type, 
            played_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        request.file_id,
        request.played_duration,
        request.total_duration,
        request.play_position,
        request.play_action.as_str(),
        request.device_type,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let play_id = result.last_insert_id() as i32;
    get_play_history_by_id(pool, play_id).await
}

// Get play history by ID
pub async fn get_play_history_by_id(
    pool: &MySqlPool,
    play_id: i32,
) -> Result<PlayHistory, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id, 
            user_id, 
            file_id, 
            played_duration, 
            total_duration, 
            play_position, 
            play_action, 
            device_type, 
            played_at
        FROM tbl_play_history
        WHERE id = ?
        "#,
        play_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(PlayHistory {
        id: row.id,
        user_id: row.user_id,
        file_id: row.file_id,
        played_duration: row.played_duration.unwrap_or(0),
        total_duration: row.total_duration,
        play_position: row.play_position,
        play_action: row.play_action,
        device_type: row.device_type,
        played_at: row.played_at.naive_utc(),
    })
}

// Get user's play history
pub async fn get_user_play_history(
    pool: &MySqlPool,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<PlayHistoryResponse>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT 
            ph.file_id, 
            f.name as file_title,
            s.name as scholar_name,
            ph.played_duration,
            ph.total_duration,
            ph.play_position,
            ph.play_action,
            ph.device_type,
            ph.played_at
        FROM tbl_play_history ph
        JOIN tbl_files f ON ph.file_id = f.id
        LEFT JOIN tbl_scholars s ON f.scholar = s.id
        WHERE ph.user_id = ?
        ORDER BY ph.played_at DESC
        LIMIT ? OFFSET ?
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let history = rows
        .into_iter()
        .map(|row| PlayHistoryResponse {
            file_id: row.file_id,
            file_title: row.file_title,
            scholar_name: row.scholar_name,
            played_duration: row.played_duration.unwrap_or(0),
            total_duration: row.total_duration,
            play_position: row.play_position,
            play_action: row.play_action,
            device_type: row.device_type,
            played_at: row.played_at.naive_utc(),
        })
        .collect();

    Ok(history)
}

// Get most played files for user
pub async fn get_user_most_played_files(
    pool: &MySqlPool,
    user_id: i32,
    limit: Option<i32>,
) -> Result<Vec<PlayHistoryResponse>, AppError> {
    let limit = limit.unwrap_or(10);

    let rows = sqlx::query!(
        r#"
        SELECT 
            ph.file_id,
            f.name as file_title,
            s.name as scholar_name,
            SUM(ph.played_duration) as total_played_duration,
            COUNT(*) as play_count,
            MAX(ph.played_at) as last_played_at
        FROM tbl_play_history ph
        JOIN tbl_files f ON ph.file_id = f.id
        LEFT JOIN tbl_scholars s ON f.scholar = s.id
        WHERE ph.user_id = ?
        GROUP BY ph.file_id, f.name, s.name
        ORDER BY total_played_duration DESC, play_count DESC
        LIMIT ?
        "#,
        user_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let history = rows
        .into_iter()
        .map(|row| PlayHistoryResponse {
            file_id: row.file_id,
            file_title: row.file_title,
            scholar_name: row.scholar_name,
            played_duration: row
                .total_played_duration
                .and_then(|bd| bd.to_string().parse::<i32>().ok())
                .unwrap_or(0),
            total_duration: None, // Aggregated data doesn't have individual total_duration
            play_position: None,  // Aggregated data doesn't have individual position
            play_action: "Aggregated".to_string(), // Indicate this is aggregated data
            device_type: None,
            played_at: row.last_played_at.unwrap().naive_utc(),
        })
        .collect();

    Ok(history)
}

// Get file play stats
pub async fn get_file_play_stats(pool: &MySqlPool, file_id: i32) -> Result<(i64, i64), AppError> {
    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_plays,
            COUNT(DISTINCT user_id) as unique_listeners
        FROM tbl_play_history
        WHERE file_id = ?
        "#,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok((row.total_plays, row.unique_listeners))
}

// Clear user play history
pub async fn clear_user_play_history(pool: &MySqlPool, user_id: i32) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM tbl_play_history WHERE user_id = ?", user_id)
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;

    Ok(())
}
