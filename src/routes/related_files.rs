use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use tracing::instrument;

use crate::{
    core::{AppConfig, AppError, AppSuccessResponse},
};

#[derive(serde::Serialize)]
pub struct RelatedFilesResponse {
    pub current_file: Option<CurrentFileInfo>,
    pub suggestions: FileSuggestions,
}

#[derive(serde::Serialize)]
pub struct CurrentFileInfo {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub book_id: i32,
    pub book_name: String,
    pub scholar_id: i32,
    pub scholar_name: String,
    pub position_in_book: Option<i32>, // Which file number in the book (1, 2, 3...)
    pub total_files_in_book: i32,
}

#[derive(serde::Serialize)]
pub struct FileSuggestions {
    pub next_in_book: Option<SimpleFileInfo>,
    pub previous_in_book: Option<SimpleFileInfo>,
    pub same_book: Vec<SimpleFileInfo>,
    pub same_scholar: Vec<SimpleFileInfo>,
    pub popular: Vec<SimpleFileInfo>,
}

#[derive(serde::Serialize)]
pub struct SimpleFileInfo {
    pub file_id: i32,
    pub file_name: String,
    pub file_url: String,
    pub file_duration: String,
    pub book_name: String,
    pub scholar_name: String,
}

#[instrument(name = "Get File Suggestions", skip(pool, config))]
#[get("/files/{file_id}/suggestions")]
pub async fn get_file_suggestions(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    file_id: web::Path<i32>,
    query: web::Query<RelatedFilesQuery>,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();
    let limit = query.limit.unwrap_or(10).min(50); // Max 50 suggestions

    // Get current file info with book and scholar details
    let current_file = get_current_file_info(&pool, &config, file_id).await?;
    
    // Get suggestions
    let suggestions = build_file_suggestions(&pool, &config, file_id, limit).await?;

    let response = RelatedFilesResponse {
        current_file,
        suggestions,
    };

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Related files retrieved successfully".to_string(),
        data: response,
        pagination: None,
    }))
}

async fn get_current_file_info(
    pool: &MySqlPool,
    config: &AppConfig,
    file_id: i32,
) -> Result<Option<CurrentFileInfo>, AppError> {
    let file_info = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.book as book_id,
            b.name as book_name,
            s.id as scholar_id,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE f.id = ? AND f.status = 'active'
        "#,
        file_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    if let Some(row) = file_info {
        // Get additional info with separate queries
        let total_files: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tbl_files WHERE book = ? AND status = 'active'",
            row.book_id
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::db_error)?;

        let position: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tbl_files WHERE book = ? AND id <= ? AND status = 'active'",
            row.book_id,
            row.file_id
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::db_error)?;

        Ok(Some(CurrentFileInfo {
            file_id: row.file_id,
            file_name: row.file_name,
            file_url: config.get_upload_url(&row.location),
            book_id: row.book_id,
            book_name: row.book_name,
            scholar_id: row.scholar_id,
            scholar_name: row.scholar_name,
            position_in_book: Some(position as i32),
            total_files_in_book: total_files as i32,
        }))
    } else {
        Ok(None)
    }
}

async fn build_file_suggestions(
    pool: &MySqlPool,
    config: &AppConfig,
    file_id: i32,
    limit: i32,
) -> Result<FileSuggestions, AppError> {
    // Get next file in same book
    let next_in_book = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration as file_duration,
            b.name as book_name,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE f.book = (SELECT book FROM tbl_files WHERE id = ?)
        AND f.id > ?
        AND f.status = 'active'
        ORDER BY f.date ASC, f.id ASC
        LIMIT 1
        "#,
        file_id,
        file_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get previous file in same book
    let previous_in_book = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration as file_duration,
            b.name as book_name,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE f.book = (SELECT book FROM tbl_files WHERE id = ?)
        AND f.id < ?
        AND f.status = 'active'
        ORDER BY f.date DESC, f.id DESC
        LIMIT 1
        "#,
        file_id,
        file_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get other files from same book
    let same_book = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration as file_duration,
            b.name as book_name,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE f.book = (SELECT book FROM tbl_files WHERE id = ?)
        AND f.id != ?
        AND f.status = 'active'
        ORDER BY f.date ASC, f.id ASC
        LIMIT ?
        "#,
        file_id,
        file_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get files from same scholar (different books)
    let same_scholar = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration as file_duration,
            b.name as book_name,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE s.id = (SELECT s2.id FROM tbl_files f2 
                      JOIN tbl_books b2 ON f2.book = b2.id 
                      JOIN tbl_scholars s2 ON b2.scholar_id = s2.id 
                      WHERE f2.id = ?)
        AND f.book != (SELECT book FROM tbl_files WHERE id = ?)
        AND f.id != ?
        AND f.status = 'active'
        ORDER BY f.downloads DESC, f.date DESC
        LIMIT ?
        "#,
        file_id,
        file_id,
        file_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get popular files (most downloaded)
    let popular = sqlx::query!(
        r#"
        SELECT 
            f.id as file_id,
            f.name as file_name,
            f.location,
            f.duration as file_duration,
            b.name as book_name,
            s.name as scholar_name
        FROM tbl_files f
        JOIN tbl_books b ON f.book = b.id
        JOIN tbl_scholars s ON b.scholar_id = s.id
        WHERE f.id != ?
        AND f.status = 'active'
        ORDER BY f.downloads DESC, f.date DESC
        LIMIT ?
        "#,
        file_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Convert to response format
    let next_in_book = next_in_book.map(|row| SimpleFileInfo {
        file_id: row.file_id,
        file_name: row.file_name,
        file_url: config.get_upload_url(&row.location),
        file_duration: row.file_duration,
        book_name: row.book_name,
        scholar_name: row.scholar_name,
    });

    let previous_in_book = previous_in_book.map(|row| SimpleFileInfo {
        file_id: row.file_id,
        file_name: row.file_name,
        file_url: config.get_upload_url(&row.location),
        file_duration: row.file_duration,
        book_name: row.book_name,
        scholar_name: row.scholar_name,
    });

    let same_book: Vec<SimpleFileInfo> = same_book.into_iter().map(|row| SimpleFileInfo {
        file_id: row.file_id,
        file_name: row.file_name,
        file_url: config.get_upload_url(&row.location),
        file_duration: row.file_duration,
        book_name: row.book_name,
        scholar_name: row.scholar_name,
    }).collect();

    let same_scholar: Vec<SimpleFileInfo> = same_scholar.into_iter().map(|row| SimpleFileInfo {
        file_id: row.file_id,
        file_name: row.file_name,
        file_url: config.get_upload_url(&row.location),
        file_duration: row.file_duration,
        book_name: row.book_name,
        scholar_name: row.scholar_name,
    }).collect();

    let popular: Vec<SimpleFileInfo> = popular.into_iter().map(|row| SimpleFileInfo {
        file_id: row.file_id,
        file_name: row.file_name,
        file_url: config.get_upload_url(&row.location),
        file_duration: row.file_duration,
        book_name: row.book_name,
        scholar_name: row.scholar_name,
    }).collect();

    Ok(FileSuggestions {
        next_in_book,
        previous_in_book,
        same_book,
        same_scholar,
        popular,
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct RelatedFilesQuery {
    pub limit: Option<i32>,
}