use crate::core::AppError;
use crate::models::files::{Files, RecentFiles};
use sqlx::MySqlPool;

pub async fn fetch_files_by_book(pool: &MySqlPool, book_id: i32) -> Result<Vec<Files>, AppError> {
    let files = sqlx::query_as!(
        Files,
        "SELECT id, name, size, duration, downloads, date, location AS url FROM tbl_files WHERE book = ?",
        book_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(files)
}

pub async fn fetch_recent_files(
    pool: &MySqlPool,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<RecentFiles>, i64), AppError> {
    // Calculate the offset
    let offset = (page - 1) * items_per_page;

    // Fetch the files with limit and offset
    let files = sqlx::query_as!(
        RecentFiles,
        r#"
        SELECT
            f.id,
            f.name as file_name,
            s.name as scholar_name,
            s.image as scholar_image,
            f.date
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        ORDER BY f.date DESC
        LIMIT ? OFFSET ?
        "#,
        items_per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Fetch total count for pagination metadata
    let total_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM tbl_files")
        .fetch_one(pool)
        .await
        .map_err(AppError::db_error)?;

    Ok((files, total_count))
}

pub async fn fetch_recent_filesssss(pool: &MySqlPool) -> Result<Vec<RecentFiles>, AppError> {
    let files = sqlx::query_as!(
        RecentFiles,
        r#"
        SELECT
            f.id,
            f.name as file_name,
            s.name as scholar_name,
            s.image as scholar_image,
            f.date
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        ORDER BY f.date DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(files)
}
