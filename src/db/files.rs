use crate::core::AppError;
use crate::models::files::{FileSearchResult, Files, RecentFiles, RelatedFiles, ViewFileDetails};
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

pub async fn search_files(
    pool: &MySqlPool,
    search_term: &str,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<FileSearchResult>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let files = sqlx::query_as!(
        FileSearchResult,
        r#"
        SELECT 
            f.id,
            f.name AS file_name,
            s.name AS scholar_name,
            CONCAT('http://yourdomain.com/images/scholars/', s.image) AS image,
            f.date
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        WHERE (f.name LIKE ? OR f.location LIKE ?) AND f.status = 'active'
        ORDER BY f.date DESC
        LIMIT ? OFFSET ?
        "#,
        format!("%{}%", search_term),
        format!("%{}%", search_term),
        items_per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    let total_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_files f
        WHERE (f.name LIKE ? OR f.location LIKE ?) AND f.status = 'active'
        "#,
        format!("%{}%", search_term),
        format!("%{}%", search_term)
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    Ok((files, total_count))
}

pub async fn fetch_file_details(
    pool: &MySqlPool,
    file_id: i32,
) -> Result<ViewFileDetails, AppError> {
    let file_details = sqlx::query_as!(
        ViewFileDetails,
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.duration,
            f.size,
            f.date as created_at,
            CONCAT('http://yourdomain.com/images/books/', b.image) as book_image
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        WHERE f.id = ? AND f.status = 'active'
        "#,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch file details: {:?}", e);
        AppError::db_error(e.to_string())
    })?;

    Ok(file_details)
}

pub async fn fetch_book_id_for_file(pool: &MySqlPool, file_id: i32) -> Result<i32, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT book
        FROM tbl_files
        WHERE id = ? AND status = 'active'
        "#,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch main file's book_id: {:?}", e);
        AppError::db_error(e.to_string())
    })?;

    Ok(result.book)
}

pub async fn fetch_related_files(
    pool: &MySqlPool,
    book_id: i32,
    exclude_file_id: i32,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<RelatedFiles>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let related_files = sqlx::query_as!(
        RelatedFiles,
        r#"
        SELECT 
            f.id,
            f.name,
            f.duration,
            f.downloads,
            f.size,
            f.date,
            CONCAT('http://yourdomain.com/files/', f.name) as url
        FROM tbl_files f
        WHERE f.book = ? AND f.id != ? AND f.status = 'active'
        ORDER BY f.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        book_id,
        exclude_file_id,
        items_per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch related files: {:?}", e);
        AppError::db_error(e.to_string())
    })?;

    let total_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_files 
        WHERE book = ? AND id != ? AND status = 'active'
        "#,
        book_id,
        exclude_file_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch total related files count: {:?}", e);
        AppError::db_error(e.to_string())
    })?;

    Ok((related_files, total_count))
}
