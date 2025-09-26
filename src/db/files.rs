use crate::core::AppError;
use crate::models::files::{
    FileSearchResult, FileStatistics, Files, FilesWithStats, RecentFiles, RecentFilesWithStats,
    RelatedFiles, ViewFileDetails,
};
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
    book_id: i32,
    pagination: &PaginationQuery,
    user_id: Option<i32>,
) -> Result<(Vec<FilesWithStats>, i64), AppError> {
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

    // Convert Files to FilesWithStats by adding statistics
    let mut files_with_stats = Vec::new();
    for file in files {
        let statistics = get_file_statistics(pool, file.file_id, user_id).await?;
        files_with_stats.push(FilesWithStats {
            file_id: file.file_id,
            file_name: file.file_name,
            file_url: file.file_url,
            file_size: file.file_size,
            book_id: file.book_id,
            file_duration: file.file_duration,
            scholar_id: file.scholar_id,
            scholar_name: file.scholar_name,
            scholar_image: file.scholar_image,
            date: file.date,
            statistics,
        });
    }

    Ok((files_with_stats, total_count))
}

pub async fn fetch_recent_files_with_stats(
    pool: &MySqlPool,
    pagination: &PaginationQuery,
    user_id: Option<i32>,
) -> Result<(Vec<RecentFilesWithStats>, i64), AppError> {
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

    // Convert RecentFiles to RecentFilesWithStats by adding statistics
    let mut files_with_stats = Vec::new();
    for file in files {
        let statistics = get_file_statistics(pool, file.file_id, user_id).await?;
        files_with_stats.push(RecentFilesWithStats {
            file_id: file.file_id,
            file_name: file.file_name,
            file_url: file.file_url,
            file_size: file.file_size,
            file_duration: file.file_duration,
            book_id: file.book_id,
            scholar_id: file.scholar_id,
            scholar_name: file.scholar_name,
            scholar_image: file.scholar_image,
            date: file.date,
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
    let files = sqlx::query!(
        r#"
        SELECT
            f.id as file_id,
            f.name as file_name,
            f.size as file_size,
            f.duration as file_duration,
            f.date,
            COALESCE(CONCAT('http://127.0.0.1:8990/api/v1/static/uploads/', f.location), '') AS file_url
        FROM tbl_files f
        WHERE f.book = ? AND f.status = 'active'
        ORDER BY f.date ASC, f.id ASC
        "#,
        book_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert to PlayAllFile structs
    let play_all_files: Vec<crate::models::files::PlayAllFile> = files
        .into_iter()
        .map(|row| crate::models::files::PlayAllFile {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: row.file_url,
            file_size: row.file_size,
            file_duration: row.file_duration,
            sort_order: None, // No sort_order column in database
            date: row.date.into(),
        })
        .collect();

    // Calculate total duration (if needed, this is optional)
    let total_duration = calculate_total_duration(&play_all_files);

    Ok(crate::models::files::PlayAllResponse {
        book_id: book_info.book_id,
        book_name: book_info.book_name,
        book_image: Some(format!(
            "http://127.0.0.1:8990/api/v1/static/images/{}",
            book_info.book_image
        )),
        scholar_id: book_info.scholar_id,
        scholar_name: book_info.scholar_name,
        scholar_image: Some(format!(
            "http://127.0.0.1:8990/api/v1/static/images/{}",
            book_info.scholar_image
        )),
        total_files: play_all_files.len() as i32,
        total_duration,
        files: play_all_files,
    })
}

// Helper function to calculate total duration
fn calculate_total_duration(files: &[crate::models::files::PlayAllFile]) -> Option<String> {
    let mut total_seconds = 0;
    let mut has_valid_duration = false;

    for file in files {
        if let Ok(duration) = parse_duration(&file.file_duration) {
            total_seconds += duration;
            has_valid_duration = true;
        }
    }

    if has_valid_duration {
        Some(format_duration(total_seconds))
    } else {
        None
    }
}

// Helper function to parse duration string (e.g., "45:30" or "1:23:45")
fn parse_duration(duration_str: &str) -> Result<u32, ()> {
    let parts: Vec<&str> = duration_str.split(':').collect();
    match parts.len() {
        2 => {
            // MM:SS format
            let minutes: u32 = parts[0].parse().map_err(|_| ())?;
            let seconds: u32 = parts[1].parse().map_err(|_| ())?;
            Ok(minutes * 60 + seconds)
        }
        3 => {
            // HH:MM:SS format
            let hours: u32 = parts[0].parse().map_err(|_| ())?;
            let minutes: u32 = parts[1].parse().map_err(|_| ())?;
            let seconds: u32 = parts[2].parse().map_err(|_| ())?;
            Ok(hours * 3600 + minutes * 60 + seconds)
        }
        _ => Err(()),
    }
}

// Helper function to format duration back to string
fn format_duration(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
