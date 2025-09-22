use crate::core::AppError;
use crate::models::access::{ScholarAccess, UserAccess, UserPermissions};
use sqlx::MySqlPool;


pub async fn fetch_user_permissions(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<UserPermissions, AppError> {
    // For now, return a default role since we don't have a users table set up yet
    // In production, you would query the actual users table
    let user_role = "Manager".to_string(); // Default role
    
    // Get accessible scholars for this user
    let scholars_data = sqlx::query!(
        r#"
        SELECT 
            s.id as scholar_id,
            s.name as scholar_name
        FROM tbl_access a
        JOIN tbl_scholars s ON a.scholar_id = s.id
        WHERE a.user_id = ? AND s.status = 'active'
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let accessible_scholars: Vec<ScholarAccess> = scholars_data
        .into_iter()
        .map(|row| ScholarAccess {
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            can_upload: true,
            can_download: true,
            can_manage: matches!(user_role.as_str(), "Admin" | "Manager"),
        })
        .collect();

    Ok(UserPermissions {
        user_id,
        accessible_scholars,
        role: user_role,
    })
}

pub async fn check_user_access_to_scholar(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_access WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(count > 0)
}

pub async fn grant_user_access(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
    created_by: i32,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    
    // First check if access already exists
    let existing_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_access WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    if existing_count > 0 {
        // Update existing record
        sqlx::query!(
            "UPDATE tbl_access SET updated_at = ? WHERE user_id = ? AND scholar_id = ?",
            now,
            user_id,
            scholar_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    } else {
        // Insert new record
        sqlx::query!(
            "INSERT INTO tbl_access (user_id, scholar_id, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            user_id,
            scholar_id,
            created_by,
            now,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    Ok(())
}

pub async fn revoke_user_access(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM tbl_access WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

pub async fn fetch_all_user_accesses(
    pool: &MySqlPool,
) -> Result<Vec<UserAccess>, AppError> {
    let access_data = sqlx::query!(
        "SELECT id, scholar_id, user_id, created_by, created_at, updated_at FROM tbl_access"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let accesses: Vec<UserAccess> = access_data
        .into_iter()
        .map(|row| UserAccess {
            id: row.id,
            scholar_id: row.scholar_id,
            user_id: row.user_id,
            created_by: row.created_by,
            created_at: row.created_at as i64,
            updated_at: row.updated_at as i64,
        })
        .collect();

    Ok(accesses)
}