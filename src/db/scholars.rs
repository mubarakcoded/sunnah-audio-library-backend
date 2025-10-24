use crate::core::{AppConfig, AppError};
use crate::models::pagination::PaginationQuery;
use crate::models::scholars::{CreateScholarRequest, Scholar, ScholarDetails, ScholarSearchResult, ScholarStatistics};
use chrono::Utc;
use sqlx::MySqlPool;

pub async fn fetch_scholars(
    pool: &MySqlPool,
    config: &AppConfig,
    pagination: &PaginationQuery,
) -> Result<(Vec<Scholar>, i64), AppError> {
    let raw_scholars = sqlx::query!(
        "SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            tbl_scholars.image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE tbl_scholars.status = 'active'
        ORDER BY tbl_scholars.priority DESC
        LIMIT ? OFFSET ?",
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert raw data to Scholar struct with formatted URLs
    let scholars: Vec<Scholar> = raw_scholars
        .into_iter()
        .map(|row| Scholar {
            id: row.id,
            name: row.name,
            image: Some(config.get_image_url(&row.image)),
            state: row.state,
        })
        .collect();

    let total_count: i64 =
        sqlx::query_scalar!("SELECT COUNT(*) FROM tbl_scholars WHERE status = 'active'")
            .fetch_one(pool)
            .await
            .map_err(AppError::db_error)?;

    Ok((scholars, total_count))
}

pub async fn fetch_scholars_by_state(
    pool: &MySqlPool,
    config: &AppConfig,
    state_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Scholar>, i64), AppError> {
    let raw_scholars = sqlx::query!(
        "SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            tbl_scholars.image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE tbl_states.id = ? AND tbl_scholars.status = 'active'
        ORDER BY tbl_scholars.priority DESC
        LIMIT ? OFFSET ?",
        state_id,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert raw data to Scholar struct with formatted URLs
    let scholars: Vec<Scholar> = raw_scholars
        .into_iter()
        .map(|row| Scholar {
            id: row.id,
            name: row.name,
            image: Some(config.get_image_url(&row.image)),
            state: row.state,
        })
        .collect();

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
    config: &AppConfig,
    search_term: &str,
    page: i32,
    items_per_page: i32,
) -> Result<(Vec<ScholarSearchResult>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let raw_scholars = sqlx::query!(
        r#"
        SELECT 
            tbl_scholars.id,
            tbl_scholars.name,
            tbl_scholars.image,
            tbl_states.name AS state
        FROM tbl_scholars
        JOIN tbl_states ON tbl_scholars.state = tbl_states.id
        WHERE (tbl_scholars.name LIKE ? ) 
        AND tbl_scholars.status = 'active'
        LIMIT ? OFFSET ?
        "#,
        format!("%{}%", search_term),
        items_per_page,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::db_error(e))?;

    // Convert raw data to ScholarSearchResult with formatted URLs
    let scholars: Vec<ScholarSearchResult> = raw_scholars
        .into_iter()
        .map(|row| ScholarSearchResult {
            id: row.id,
            name: row.name,
            image: Some(config.get_image_url(&row.image)),
            state: Some(row.state),
        })
        .collect();

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
    config: &AppConfig,
    scholar_id: i32,
    user_id: Option<i32>,
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

    // Check if user follows this scholar (if user_id is provided)
    let is_followed_by_user = if let Some(uid) = user_id {
        check_user_follows_scholar(pool, uid, scholar_id).await?
    } else {
        None
    };

    // Check if user has access to this scholar (for managers)
    let has_access = if let Some(uid) = user_id {
        check_user_has_scholar_access(pool, uid, scholar_id).await?
    } else {
        None
    };

    Ok(ScholarDetails {
        id: scholar_row.id,
        name: scholar_row.name,
        about: Some(scholar_row.about),
        state: scholar_row.state_name,
        image: Some(config.get_image_url(&scholar_row.image)),
        created_at: Utc::now().naive_utc(), // Using current time as placeholder
        updated_at: Utc::now().naive_utc(), // Using current time as placeholder
        statistics,
        is_followed_by_user,
        has_access,
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

pub async fn check_user_follows_scholar(
    pool: &MySqlPool,
    user_id: i32,
    scholar_id: i32,
) -> Result<Option<bool>, AppError> {
    let follow_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_user_scholar_follows WHERE user_id = ? AND scholar_id = ?",
        user_id,
        scholar_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(Some(follow_count > 0))
}

pub async fn check_user_has_scholar_access(
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

pub async fn get_scholars_dropdown(
    pool: &MySqlPool,
) -> Result<Vec<crate::models::scholars::ScholarDropdown>, AppError> {
    let scholars = sqlx::query!(
        "SELECT id, name FROM tbl_scholars WHERE status = 'active' ORDER BY name"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let dropdown_scholars = scholars
        .into_iter()
        .map(|row| crate::models::scholars::ScholarDropdown {
            id: row.id,
            name: row.name,
        })
        .collect();

    Ok(dropdown_scholars)
}

pub async fn create_scholar(
    pool: &MySqlPool,
    request: &CreateScholarRequest,
    user_id: i32,
    slug_value: &str,
) -> Result<i32, AppError> {

    let about_value: String = request.about.clone().unwrap_or_default();
    let image_value: &str = request.image.as_deref().unwrap_or("scholar.jpg");
    let now = Utc::now().naive_utc();


    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_scholars (name, about, state, image, slug, status, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?)
        "#,
        request.name,
        about_value,
        request.state_id,
        image_value,
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

pub async fn update_scholar(
    pool: &MySqlPool,
    scholar_id: i32,
    request: &crate::models::scholars::UpdateScholarRequest,
) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();

    // Update each field individually if provided
    if let Some(ref name) = request.name {
        sqlx::query!(
            "UPDATE tbl_scholars SET name = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            name,
            now,
            scholar_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(ref about) = request.about {
        sqlx::query!(
            "UPDATE tbl_scholars SET about = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            about,
            now,
            scholar_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(state_id) = request.state_id {
        sqlx::query!(
            "UPDATE tbl_scholars SET state = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            state_id,
            now,
            scholar_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(ref image) = request.image {
        sqlx::query!(
            "UPDATE tbl_scholars SET image = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            image,
            now,
            scholar_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    Ok(())
}

pub async fn check_duplicate_scholar(
    pool: &MySqlPool,
    name: &str,
    slug_value: &str,
) -> Result<Option<String>, AppError> {
    
    let existing = sqlx::query!(
        r#"
        SELECT name FROM tbl_scholars 
        WHERE name = ? OR slug = ?
        LIMIT 1
        "#,
        name,
        slug_value
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(existing.map(|s| s.name))
}