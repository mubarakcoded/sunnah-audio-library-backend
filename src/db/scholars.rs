use crate::core::AppError;
use crate::models::pagination::PaginationQuery;
use crate::models::scholars::{Scholar, ScholarSearchResult};
use sqlx::MySqlPool;

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
    page: i64,
    items_per_page: i64,
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
