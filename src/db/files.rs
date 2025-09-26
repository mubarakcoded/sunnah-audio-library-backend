use crate::core::AppError;
use crate::models::files::{FileSearchResult, Files, RecentFiles, RelatedFiles, ViewFileDetails};
use crate::models::pagination::PaginationQuery;
use sqlx::MySqlPool;

pub async fn fetch_files_by_book(
    pool: &MySqlPool,
    book_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Files>, i64), AppError> {
    let files = sqlx::query_as!(
        Files,
        "SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/uploads/', f.location), '') AS file_url,
            s.id as scholar_id,
            s.name as scholar_name,
            COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/images/', s.image), '') AS scholar_image
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        WHERE f.status = 'active'
        AND f.book = ?
        LIMIT ? OFFSET ?",
        book_id,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let total_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_files WHERE book = ? AND status = 'active'",
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok((files, total_count))
}

pub async fn fetch_recent_files(
    pool: &MySqlPool,
    pagination: &PaginationQuery,
) -> Result<(Vec<RecentFiles>, i64), AppError> {
    let files = sqlx::query_as!(
        RecentFiles,
        r#"
        SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/uploads/', f.location), '') AS "file_url!",
            s.id as scholar_id,
            s.name as scholar_name,
            COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/images/', s.image), '') AS "scholar_image!"
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        ORDER BY f.date DESC
        LIMIT ? OFFSET ?
        "#,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let total_count: i64 =
        sqlx::query_scalar!("SELECT COUNT(*) FROM tbl_files WHERE status = 'active'")
            .fetch_one(pool)
            .await
            .map_err(AppError::db_error)?;

    Ok((files, total_count))
}

pub async fn search_files(
    pool: &MySqlPool,
    search_term: &str,
    page: i32,
    items_per_page: i32,
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

pub async fn create_file_record(
    pool: &MySqlPool,
    name: &str,
    location: &str,
    size: i32,
    duration: Option<f64>,
    book_id: i32,
    scholar_id: i32,
) -> Result<i32, AppError> {
    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_files (name, location, size, duration, book, scholar, status, created_at, date)
        VALUES (?, ?, ?, ?, ?, ?, 'active', NOW(), NOW())
        "#,
        name,
        location,
        size,
        duration,
        book_id,
        scholar_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create file record: {:?}", e);
        AppError::db_error(e.to_string())
    })?;

    Ok(result.last_insert_id() as i32)
}
