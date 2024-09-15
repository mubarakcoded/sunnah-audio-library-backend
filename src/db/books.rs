use crate::core::AppError;
use crate::models::books::Book;
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
