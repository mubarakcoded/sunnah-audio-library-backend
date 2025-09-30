use crate::core::{AppError, AppConfig};
use crate::models::books::{Book, BookSearchResult, BookDetails, BookStatistics};
use crate::models::pagination::PaginationQuery;
use sqlx::MySqlPool;
use chrono::Utc;

pub async fn fetch_books_by_scholar(
    pool: &MySqlPool,
    config: &AppConfig,
    scholar_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Book>, i64), AppError> {
    let raw_books = sqlx::query!(
        "SELECT
        id,
        name,
        image,
        created_at,
        created_by
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

    // Convert raw data to Book struct with formatted URLs
    let books: Vec<Book> = raw_books
        .into_iter()
        .map(|row| Book {
            id: row.id,
            name: row.name,
            image: config.get_image_url(&row.image),
            created_at: row.created_at.naive_utc(),
            created_by: row.created_by
        })
        .collect();

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
    config: &AppConfig,
    search_term: &str,
    page: i32,
    per_page: i32,
) -> Result<(Vec<BookSearchResult>, i64), AppError> {
    let offset = (page - 1) * per_page;

    let raw_books = sqlx::query!(
        r#"
        SELECT 
            b.id,
            b.name,
            b.image
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

    // Convert raw data to BookSearchResult with formatted URLs
    let books: Vec<BookSearchResult> = raw_books
        .into_iter()
        .map(|row| BookSearchResult {
            id: row.id,
            name: Some(row.name),
            image: Some(config.get_image_url(&row.image)),
        })
        .collect();

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
    config: &AppConfig,
    book_id: i32,
    user_id: Option<i32>,
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

    // Check if user has access to this book's scholar (for managers)
    let has_access = if let Some(uid) = user_id {
        check_user_has_book_access(pool, uid, book_row.scholar_id).await?
    } else {
        None
    };

    Ok(BookDetails {
        id: book_row.id,
        name: book_row.name,
        about: Some(book_row.about),
        scholar_id: book_row.scholar_id,
        scholar_name: book_row.scholar_name,
        image: Some(config.get_image_url(&book_row.image)),
        created_at: Utc::now().naive_utc(), // Using current time as placeholder
        updated_at: Utc::now().naive_utc(), // Using current time as placeholder
        statistics,
        has_access,
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

pub async fn check_user_has_book_access(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<Option<bool>, AppError> {
    // First check if user is admin
    let user_role = sqlx::query_scalar!(
        "SELECT role FROM tbl_users WHERE id = ?",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    if user_role == "admin" {
        return Ok(Some(true));
    }

    // Check if user has specific access to this scholar
    let access_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_access WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(Some(access_count > 0))
}

pub async fn get_books_dropdown(
    pool: &MySqlPool,
    scholar_id: Option<i32>,
) -> Result<Vec<crate::models::books::BookDropdown>, AppError> {
    let books = if let Some(sid) = scholar_id {
        sqlx::query_as!(
            crate::models::books::BookDropdown,
            r#"
            SELECT b.id, b.name, b.scholar_id, s.name as scholar_name
            FROM tbl_books b
            JOIN tbl_scholars s ON b.scholar_id = s.id
            WHERE b.scholar_id = ? AND b.status = 'active' AND s.status = 'active'
            ORDER BY b.name
            "#,
            sid
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::db_error)?
    } else {
        sqlx::query_as!(
            crate::models::books::BookDropdown,
            r#"
            SELECT b.id, b.name, b.scholar_id, s.name as scholar_name
            FROM tbl_books b
            JOIN tbl_scholars s ON b.scholar_id = s.id
            WHERE b.status = 'active' AND s.status = 'active'
            ORDER BY s.name, b.name
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::db_error)?
    };

    Ok(books)
}

pub async fn create_book(
    pool: &MySqlPool,
    request: &crate::models::books::CreateBookRequest,
    slug_value: &str,
    user_id: i32,
) -> Result<i32, AppError> {
    let now = Utc::now().naive_utc();
    
    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_books (name, about, scholar_id, image, slug, status, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?)
        "#,
        request.name,
        request.about,
        request.scholar_id,
        request.image.as_deref().unwrap_or("book.jpg"),
        slug_value,
        user_id,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(result.last_insert_id() as i32)
}

pub async fn update_book(
    pool: &MySqlPool,
    book_id: i32,
    request: &crate::models::books::UpdateBookRequest,
) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();

    // Update each field individually if provided
    if let Some(ref name) = request.name {
        sqlx::query!(
            "UPDATE tbl_books SET name = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            name,
            now,
            book_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(ref about) = request.about {
        sqlx::query!(
            "UPDATE tbl_books SET about = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            about,
            now,
            book_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(scholar_id) = request.scholar_id {
        sqlx::query!(
            "UPDATE tbl_books SET scholar_id = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            scholar_id,
            now,
            book_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(ref image) = request.image {
        sqlx::query!(
            "UPDATE tbl_books SET image = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            image,
            now,
            book_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    Ok(())
}

pub async fn check_duplicate_book(
    pool: &MySqlPool,
    name: &str,
    scholar_id: i32,
    slug_value: &str,
) -> Result<Option<String>, AppError> {
    
    let existing = sqlx::query!(
        r#"
        SELECT name FROM tbl_books 
        WHERE (name = ? OR slug = ?) AND scholar_id = ?
        LIMIT 1
        "#,
        name,
        slug_value,
        scholar_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(existing.map(|b| b.name))
}