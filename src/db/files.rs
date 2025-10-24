use crate::core::{calculate_total_duration_from_strings, AppConfig, AppError};
use crate::models::files::{
    FileSearchResult, FileStatistics, Files, FilesWithStats, RecentFiles, RecentFilesWithStats,
    RelatedFiles, ViewFileDetails,
};
use crate::models::pagination::PaginationQuery;
use sqlx::MySqlPool;

pub async fn fetch_files_by_book(
    pool: &MySqlPool,
    config: &AppConfig,
    book_id: i32,
    pagination: &PaginationQuery,
) -> Result<(Vec<Files>, i64), AppError> {
    let raw_files = sqlx::query!(
        "SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            f.location,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
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

    // Convert raw data to Files struct with formatted URLs
    let files: Vec<Files> = raw_files
        .into_iter()
        .map(|row| Files {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            book_id: row.book_id,
            file_duration: row.file_duration,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            scholar_image: config.get_image_url(&row.scholar_image),
            date: row.date.into(),
            downloads: row.downloads,
        })
        .collect();

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
    config: &AppConfig,
    pagination: &PaginationQuery,
) -> Result<(Vec<RecentFiles>, i64), AppError> {
    let raw_files = sqlx::query!(
        r#"
        SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            f.location,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        WHERE f.status = 'active'
        ORDER BY f.date DESC
        LIMIT ? OFFSET ?
        "#,
        pagination.per_page,
        pagination.offset()
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert raw data to RecentFiles struct with formatted URLs
    let files: Vec<RecentFiles> = raw_files
        .into_iter()
        .map(|row| RecentFiles {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            file_duration: row.file_duration,
            downloads: row.downloads,
            book_id: row.book_id,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            scholar_image: config.get_image_url(&row.scholar_image),
            date: row.date.into(),
        })
        .collect();

    let total_count: i64 =
        sqlx::query_scalar!("SELECT COUNT(*) FROM tbl_files WHERE status = 'active'")
            .fetch_one(pool)
            .await
            .map_err(AppError::db_error)?;

    Ok((files, total_count))
}

pub async fn search_files(
    pool: &MySqlPool,
    config: &AppConfig,
    search_term: &str,
    page: i32,
    items_per_page: i32,
) -> Result<(Vec<FileSearchResult>, i64), AppError> {
    let offset = (page - 1) * items_per_page;

    let raw_files = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.downloads,
            f.location,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
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

    // Convert raw data to FileSearchResult struct with formatted URLs
    let files: Vec<FileSearchResult> = raw_files
        .into_iter()
        .map(|row| FileSearchResult {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            file_duration: row.file_duration,
            downloads: row.downloads,
            book_id: row.book_id,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            scholar_image: config.get_image_url(&row.scholar_image),
        })
        .collect();

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
    config: &AppConfig,
    file_id: i32,
) -> Result<ViewFileDetails, AppError> {
    let raw_file = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration,
            f.size,
            f.date as created_at,
            f.downloads,
            b.id as book_id,
            b.name as book_name,
            b.image as book_image,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON f.scholar = s.id
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

    Ok(ViewFileDetails {
        file_id: raw_file.file_id,
        file_name: raw_file.file_name,
        file_url: config.get_upload_url(&raw_file.location),
        duration: raw_file.duration,
        size: raw_file.size,
        created_at: raw_file.created_at.into(),
        book_id: raw_file.book_id,
        book_name: raw_file.book_name,
        book_image: Some(config.get_image_url(&raw_file.book_image)),
        scholar_id: raw_file.scholar_id,
        scholar_name: raw_file.scholar_name,
        scholar_image: config.get_image_url(&raw_file.scholar_image),
        downloads: raw_file.downloads,
    })
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
    pagination: &PaginationQuery,
) -> Result<(Vec<RelatedFiles>, i64), AppError> {
    // let offset = (page - 1) * items_per_page;

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
        pagination.per_page,
        pagination.offset()
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
pub async fn fetch_files_by_book_with_stats(
    pool: &MySqlPool,
    config: &AppConfig,
    book_id: i32,
    pagination: &PaginationQuery,
    user_id: Option<i32>,
) -> Result<(Vec<FilesWithStats>, i64), AppError> {
    let raw_files = sqlx::query!(
        "SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            f.location,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
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

    // Convert raw data to FilesWithStats by adding statistics and formatting URLs
    let mut files_with_stats = Vec::new();
    for row in raw_files {
        let statistics = get_file_statistics(pool, row.file_id, user_id).await?;
        files_with_stats.push(FilesWithStats {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            book_id: row.book_id,
            file_duration: row.file_duration,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            scholar_image: config.get_image_url(&row.scholar_image),
            date: row.date.into(),
            statistics,
        });
    }

    Ok((files_with_stats, total_count))
}

pub async fn fetch_recent_files_with_stats(
    pool: &MySqlPool,
    config: &AppConfig,
    pagination: &PaginationQuery,
    user_id: Option<i32>,
) -> Result<(Vec<RecentFilesWithStats>, i64), AppError> {
    let raw_files = sqlx::query!(
        r#"
        SELECT
            f.id as file_id,
            f.name as file_name,
            f.book as book_id,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.downloads,
            f.location,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
        FROM tbl_files f
        JOIN tbl_scholars s ON f.scholar = s.id
        WHERE f.status = 'active'
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

    // Convert raw data to RecentFilesWithStats by adding statistics and formatting URLs
    let mut files_with_stats = Vec::new();
    for row in raw_files {
        let statistics = get_file_statistics(pool, row.file_id, user_id).await?;
        files_with_stats.push(RecentFilesWithStats {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            file_duration: row.file_duration,
            book_id: row.book_id,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            scholar_image: config.get_image_url(&row.scholar_image),
            date: row.date.into(),
            statistics,
        });
    }

    Ok((files_with_stats, total_count))
}

pub async fn get_file_statistics(
    pool: &MySqlPool,
    file_id: i32,
    user_id: Option<i32>,
) -> Result<FileStatistics, AppError> {
    // Get total downloads
    let total_downloads: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_download_logs WHERE file_id = ?",
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total plays
    let total_plays: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_play_history WHERE file_id = ?",
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total likes
    let total_likes: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_file_likes WHERE file_id = ?",
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get total comments
    let total_comments: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_file_comments WHERE file_id = ? AND is_approved = 1",
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Check if user has liked this file (if user_id is provided)
    let is_liked_by_user = if let Some(uid) = user_id {
        let like_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tbl_file_likes WHERE file_id = ? AND user_id = ?",
            file_id,
            uid
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::db_error)?;

        Some(like_count > 0)
    } else {
        None
    };

    Ok(FileStatistics {
        total_downloads,
        total_plays,
        total_likes,
        total_comments,
        is_liked_by_user,
    })
}

pub async fn get_all_files_for_book_play_all(
    pool: &MySqlPool,
    config: &AppConfig,
    book_id: i32,
) -> Result<crate::models::files::PlayAllResponse, AppError> {
    // First, get book and scholar information
    let book_info = sqlx::query!(
        r#"
        SELECT 
            b.id as book_id,
            b.name as book_name,
            b.image as book_image,
            s.id as scholar_id,
            s.name as scholar_name,
            s.image as scholar_image
        FROM tbl_books b
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE b.id = ? AND b.status = 'active' AND s.status = 'active'
        "#,
        book_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get all files for the book, ordered by creation date
    let raw_files = sqlx::query!(
        r#"
        SELECT
            f.id as file_id,
            f.name as file_name,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            f.location
        FROM tbl_files f
        WHERE f.book = ? AND f.status = 'active'
        ORDER BY f.date ASC, f.id ASC
        "#,
        book_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert to PlayAllFile structs with formatted URLs
    let play_all_files: Vec<crate::models::files::PlayAllFile> = raw_files
        .into_iter()
        .map(|row| crate::models::files::PlayAllFile {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            file_size: row.file_size,
            file_duration: row.file_duration,
            sort_order: None, // No sort_order column in database
            date: row.date.into(),
        })
        .collect();

    // Calculate total duration (if needed, this is optional)
    let duration_strings: Vec<String> = play_all_files
        .iter()
        .map(|f| f.file_duration.clone())
        .collect();
    let total_duration = calculate_total_duration_from_strings(&duration_strings);

    Ok(crate::models::files::PlayAllResponse {
        book_id: book_info.book_id,
        book_name: book_info.book_name,
        book_image: Some(config.get_image_url(&book_info.book_image)),
        scholar_id: book_info.scholar_id,
        scholar_name: book_info.scholar_name,
        scholar_image: Some(config.get_image_url(&book_info.scholar_image)),
        total_files: play_all_files.len() as i32,
        total_duration,
        files: play_all_files,
    })
}

pub async fn update_file(
    pool: &MySqlPool,
    file_id: i32,
    request: &crate::models::files::UpdateFileRequest,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().naive_utc();

    // Update each field individually if provided
    if let Some(ref title) = request.name {
        sqlx::query!(
            "UPDATE tbl_files SET name = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            title,
            now,
            file_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    if let Some(book_id) = request.book_id {
        sqlx::query!(
            "UPDATE tbl_files SET book = ?, updated_at = ? WHERE id = ? AND status = 'active'",
            book_id,
            now,
            file_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    Ok(())
}

pub async fn check_file_owner_or_admin(
    pool: &MySqlPool,
    user_id: i32,
    _file_id: i32,
) -> Result<bool, AppError> {
    // First check if user is admin
    let user_role = sqlx::query_scalar!(
        "SELECT role FROM tbl_users WHERE id = ?",
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    if user_role == "admin" {
        return Ok(true);
    }

    // For now, since uploaded_by column doesn't exist, only allow admins
    // In production, you should add uploaded_by column to track file ownership
    Ok(false)
}