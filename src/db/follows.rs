use crate::core::AppError;
use crate::models::follows::{UserScholarFollow, FollowScholarRequest, UpdateFollowRequest, FollowResponse};
use sqlx::MySqlPool;
use chrono::Utc;

// Follow a scholar
pub async fn follow_scholar(
    pool: &MySqlPool,
    user_id: i32,
    request: &FollowScholarRequest,
) -> Result<UserScholarFollow, AppError> {
    let now = Utc::now().naive_utc();
    let notifications_enabled = request.notifications_enabled.unwrap_or(true);

    let _result = sqlx::query!(
        r#"
        INSERT INTO tbl_user_scholar_follows (user_id, scholar_id, notifications_enabled, followed_at)
        VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE 
            notifications_enabled = VALUES(notifications_enabled),
            followed_at = VALUES(followed_at)
        "#,
        user_id,
        request.scholar_id,
        notifications_enabled,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_user_follow(pool, user_id, request.scholar_id).await
}

// Unfollow a scholar
pub async fn unfollow_scholar(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM tbl_user_scholar_follows WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

// Update follow settings
pub async fn update_follow_settings(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
    request: &UpdateFollowRequest,
) -> Result<UserScholarFollow, AppError> {
    sqlx::query!(
        "UPDATE tbl_user_scholar_follows SET notifications_enabled = ? WHERE user_id = ? AND scholar_id = ?",
        request.notifications_enabled,
        user_id,
        scholar_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_user_follow(pool, user_id, scholar_id).await
}

// Get user's follow for a specific scholar
pub async fn get_user_follow(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<UserScholarFollow, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, scholar_id, notifications_enabled, followed_at
        FROM tbl_user_scholar_follows
        WHERE user_id = ? AND scholar_id = ?
        "#,
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(UserScholarFollow {
        id: row.id,
        user_id: row.user_id,
        scholar_id: row.scholar_id,
        notifications_enabled: row.notifications_enabled.unwrap_or(0) != 0,
        followed_at: row.followed_at.naive_utc(),
    })
}

// Get user's followed scholars
pub async fn get_user_followed_scholars(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<FollowResponse>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT f.scholar_id, s.name as scholar_name, f.notifications_enabled, f.followed_at
        FROM tbl_user_scholar_follows f
        JOIN tbl_scholars s ON f.scholar_id = s.id
        WHERE f.user_id = ?
        ORDER BY f.followed_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let follows = rows
        .into_iter()
        .map(|row| FollowResponse {
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            notifications_enabled: row.notifications_enabled.unwrap_or(0) != 0,
            followed_at: row.followed_at.naive_utc(),
        })
        .collect();

    Ok(follows)
}

// Check if user follows scholar
pub async fn is_following_scholar(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<bool, AppError> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM tbl_user_scholar_follows WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(count.count > 0)
}

// Get scholar followers count
pub async fn get_scholar_followers_count(
    pool: &MySqlPool,
    scholar_id: i32,
) -> Result<i64, AppError> {
    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM tbl_user_scholar_follows WHERE scholar_id = ?",
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(row.count)
}