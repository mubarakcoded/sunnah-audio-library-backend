use crate::core::AppError;
use crate::models::books::{Book, BookSearchResult, BookDetails, BookStatistics};
use crate::models::pagination::PaginationQuery;
use sqlx::MySqlPool;
use chrono::Utc;

pub async fn fetch_books_by_scholar(
    pool: &MySqlPool,
    scholar_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Book>, i64), AppError> {
    let books = sqlx::query_as!(
        Book,
        "SELECT
        id,
        name,
        COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/images/', image), '') AS image
        FROM tbl_books 
        WHERE scholar_id = ? AND status = 'active'
        LIMIT ? OFFSET ?",
        scholar_id,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let total_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_books WHERE scholar_id = ? AND status = 'active'",
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok((books, total_count))
}

pub async fn search_books(
    pool: &MySqlPool,
    search_term: &str,
    page: i32,
    per_page: i32,
) -> Result<(Vec<BookSearchResult>, i64), AppError> {
    let offset = (page - 1) * per_page;

    let books = sqlx::query_as!(
        BookSearchResult,
        r#"
        SELECT 
            b.id,
            b.name,
            CONCAT('http://127.0.0.1:8990/api/v1/static/images/', b.image) AS image
        FROM tbl_books b
        WHERE (b.name LIKE ? OR b.about LIKE ?) AND b.status = 'active'
        LIMIT ? OFFSET ?
        "#,
        format!("%{}%", search_term),
        format!("%{}%", search_term),
        per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    let total_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_books 
        WHERE (name LIKE ? OR about LIKE ?) AND status = 'active'
        "#,
        format!("%{}%", search_term),
        format!("%{}%", search_term)
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    Ok((books, total_count))
}
pub async fn get_book_details(
    pool: &MySqlPool,
    book_id: i32,
) -> Result<BookDetails, AppError> {
    // Get basic book information with scholar details
    let book_row = sqlx::query!(
        r#"
        SELECT 
            b.id, b.name, b.about, b.scholar_id, b.image, b.created_at, b.updated_at,
            s.name as scholar_name
        FROM tbl_books b
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE b.id = ? AND b.status = 'active' AND s.status = 'active'
        "#,
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get statistics
    let statistics = get_book_statistics(pool, book_id).await?;

    Ok(BookDetails {
        id: book_row.id,
        name: book_row.name,
        about: Some(book_row.about),
        scholar_id: book_row.scholar_id,
        scholar_name: book_row.scholar_name,
        image: Some(format!("http://127.0.0.1:8990/api/v1/static/images/{}", book_row.image)),
        created_at: Utc::now().naive_utc(), // Using current time as placeholder
        updated_at: Utc::now().naive_utc(), // Using current time as placeholder
        statistics,
    })
}

pub async fn get_book_statistics(
    pool: &MySqlPool,
    book_id: i32,
) -> Result<BookStatistics, AppError> {
    // Get total files
    let total_files: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_files WHERE book = ? AND status = 'active'",
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total downloads
    let total_downloads: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_download_logs dl
        JOIN tbl_files f ON dl.file_id = f.id
        WHERE f.book = ?
        "#,
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total plays
    let total_plays: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_play_history ph
        JOIN tbl_files f ON ph.file_id = f.id
        WHERE f.book = ?
        "#,
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total likes
    let total_likes: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_file_likes fl
        JOIN tbl_files f ON fl.file_id = f.id
        WHERE f.book = ?
        "#,
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get average rating (if you have a rating system)
    // For now, setting to None as there's no rating table visible
    let average_rating: Option<f64> = None;

    Ok(BookStatistics {
        total_files,
        total_downloads,
        total_plays,
        total_likes,
        average_rating,
    })
}