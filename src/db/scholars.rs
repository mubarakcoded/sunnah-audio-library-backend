use crate::core::AppError;
use crate::models::pagination::PaginationQuery;
use crate::models::scholars::{Scholar, ScholarSearchResult, ScholarDetails, ScholarStatistics};
use sqlx::MySqlPool;
use chrono::Utc;

pub async fn fetch_scholars(
    pool: &MySqlPool,
    pagination: &PaginationQuery,
) -> Result<(Vec<Scholar>, i64), AppError> {
    let scholars = sqlx::query_as!(
        Scholar,
        "SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            CONCAT('http://127.0.0.1:8990/api/v1/static/images/', tbl_scholars.image) AS image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE tbl_scholars.status = 'active'
        LIMIT ? OFFSET ?",
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let total_count: i64 =
        sqlx::query_scalar!("SELECT COUNT(*) FROM tbl_scholars WHERE status = 'active'")
            .fetch_one(pool)
            .await
            .map_err(AppError::db_error)?;

    Ok((scholars, total_count))
}

pub async fn fetch_scholars_by_state(
    pool: &MySqlPool,
    state_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Scholar>, i64), AppError> {
    let scholars = sqlx::query_as!(
        Scholar,
        "SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            CONCAT('http://127.0.0.1:8990/api/v1/static/images/', tbl_scholars.image) AS image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE tbl_states.id = ? AND tbl_scholars.status = 'active'
        LIMIT ? OFFSET ?",
        state_id,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let total_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_scholars WHERE state = ? AND status = 'active'",
        state_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok((scholars, total_count))
}

pub async fn search_scholars(
    pool: &MySqlPool,
    search_term: &str,
    page: i32,
    items_per_page: i32,
) -> Result<(Vec<ScholarSearchResult>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let scholars = sqlx::query_as!(
        ScholarSearchResult,
        r#"
        SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            CONCAT('http://yourdomain.com/images/scholars/', tbl_scholars.image) AS image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE (tbl_scholars.name LIKE ? OR tbl_scholars.about LIKE ?) 
        AND tbl_scholars.status = 'active'
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
        FROM tbl_scholars 
        WHERE (name LIKE ? OR about LIKE ?) AND status = 'active'
        "#,
        format!("%{}%", search_term),
        format!("%{}%", search_term)
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    Ok((scholars, total_count))
}

pub async fn get_scholar_details(
    pool: &MySqlPool,
    scholar_id: i32,
) -> Result<ScholarDetails, AppError> {
    // Get basic scholar information
    let scholar_row = sqlx::query!(
        r#"
        SELECT 
            s.id, s.name, s.about, s.image, s.created_at, s.updated_at,
            st.name as state_name
        FROM tbl_scholars s
        JOIN tbl_states st ON s.state = st.id
        WHERE s.id = ? AND s.status = 'active'
        "#,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get statistics
    let statistics = get_scholar_statistics(pool, scholar_id).await?;

    Ok(ScholarDetails {
        id: scholar_row.id,
        name: scholar_row.name,
        about: Some(scholar_row.about),
        state: scholar_row.state_name,
        image: Some(format!("http://127.0.0.1:8990/api/v1/static/images/{}", scholar_row.image)),
        created_at: Utc::now().naive_utc(), // Using current time as placeholder
        updated_at: Utc::now().naive_utc(), // Using current time as placeholder
        statistics,
    })
}

pub async fn get_scholar_statistics(
    pool: &MySqlPool,
    scholar_id: i32,
) -> Result<ScholarStatistics, AppError> {
    // Get total books
    let total_books: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_books WHERE scholar_id = ? AND status = 'active'",
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total files
    let total_files: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) 
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        WHERE b.scholar_id = ? AND f.status = 'active' AND b.status = 'active'
        "#,
        scholar_id
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
        JOIN tbl_books b ON f.book = b.id
        WHERE b.scholar_id = ?
        "#,
        scholar_id
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
        JOIN tbl_books b ON f.book = b.id
        WHERE b.scholar_id = ?
        "#,
        scholar_id
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
        JOIN tbl_books b ON f.book = b.id
        WHERE b.scholar_id = ?
        "#,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total followers
    let total_followers: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_user_scholar_follows WHERE scholar_id = ?",
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(ScholarStatistics {
        total_books,
        total_files,
        total_downloads,
        total_plays,
        total_likes,
        total_followers,
    })
}