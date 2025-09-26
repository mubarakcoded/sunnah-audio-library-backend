use crate::core::AppError;
use crate::models::books::{Book, BookSearchResult};
use crate::models::pagination::PaginationQuery;
use sqlx::MySqlPool;

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
