use crate::core::AppError;
use crate::models::books::{Book, BookSearchResult};
use sqlx::MySqlPool;

pub async fn fetch_books_by_scholar(
    pool: &MySqlPool,
    scholar_id: i32,
) -> Result<Vec<Book>, AppError> {
    let books = sqlx::query_as!(
        Book,
        "SELECT id, name, image FROM tbl_books WHERE scholar_id = ?",
        scholar_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(books)
}

pub async fn search_books(
    pool: &MySqlPool,
    search_term: &str,
    page: i64,
    items_per_page: i64,
) -> Result<(Vec<BookSearchResult>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let books = sqlx::query_as!(
        BookSearchResult,
        r#"
        SELECT 
            b.id,
            b.name,
            CONCAT('http://yourdomain.com/images/books/', b.image) AS image
        FROM tbl_books b
        WHERE (b.name LIKE ? OR b.about LIKE ?) AND b.status = 'active'
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
