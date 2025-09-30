use crate::core::AppError;
use crate::models::uploads::{FileDownloadInfo, FileUploadResponse};
use sqlx::MySqlPool;


pub async fn save_uploaded_file(
    pool: &MySqlPool,
    book_id: i32,        // Extracted from MP3
    filename: &str,
    file_path: &str,
    file_size: i64,
    content_type: &str,
    duration: &str,        // Formatted duration (MM:SS or HH:MM:SS)
    random_id: &str,
    user_id: i32,
) -> Result<FileUploadResponse, AppError> {
    let now = chrono::Utc::now();

    // Get scholar_id from book_id first
    let scholar_id = get_scholar_id_from_book(pool, book_id).await?;

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_files 
        (book, scholar, name, location, size, type, duration, uid, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        book_id,
        scholar_id,
        filename,
        file_path,
        file_size,
        content_type,
        duration,
        random_id,
        user_id,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let file_id = result.last_insert_id() as i32;

    Ok(FileUploadResponse {
        file_id,
        filename: filename.to_string(),
        file_path: file_path.to_string(),
        file_size,
        content_type: content_type.to_string(),
    })
}

pub async fn get_file_download_info(
    pool: &MySqlPool,
    file_id: i32,
) -> Result<FileDownloadInfo, AppError> {
    let file_data = sqlx::query!(
        r#"
        SELECT 
            f.id,
            f.name,
            f.location,
            f.size,
            f.book,
            f.scholar
        FROM tbl_files f
        WHERE f.id = ? AND f.status = 'active'
        "#,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    let file_info = FileDownloadInfo {
        file_id: file_data.id,
        filename: file_data.name,
        file_path: format!("./uploads/{}", file_data.location),
        content_type: "application/octet-stream".to_string(), // Default since not stored
        file_size: file_data.size.parse().unwrap_or(0),
        book_id: file_data.book,
        scholar_id: file_data.scholar,
    };

    Ok(file_info)
}

pub async fn check_file_access_permission(
    pool: &MySqlPool,
    user_id: i32,
    file_id: i32,
) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_access a ON b.scholar_id = a.scholar_id
        WHERE f.id = ? AND a.user_id = ? AND f.status = 'active'
        "#,
        file_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(count > 0)
}

pub async fn get_scholar_id_from_book(pool: &MySqlPool, book_id: i32) -> Result<i32, AppError> {
    let scholar_id: i32 = sqlx::query_scalar!(
        "SELECT scholar_id FROM tbl_books WHERE id = ? AND status = 'active'",
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(scholar_id)
}
