use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::TryStreamExt;
use sqlx::MySqlPool;
use std::fs;
use std::path::Path;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    core::{
        extract_mp3_metadata, jwt_auth::JwtMiddleware, AppError, AppErrorType, AppSuccessResponse,
    },
    db::{access, file_interactions, subscriptions, uploads},
};

// const UPLOAD_DIR: &str = "./uploads";
// const UPLOAD_DIR: &str = "/home/mubarak/Documents/my-documents/muryar_sunnah/web/uploads";

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

#[instrument(name = "Upload File", skip(pool, payload))]
#[post("/{book_id}/upload")]
pub async fn upload_file(
    pool: web::Data<MySqlPool>,
    config: web::Data<crate::core::config::AppConfig>,
    auth: JwtMiddleware,
    book_id: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    // Check if user has access to upload to this book's scholar

    let user = crate::db::users::get_user_by_id(pool.get_ref(), auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            AppError {
                message: Some("User not found".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::NotFoundError,
            }
        })?;

    if user.role != "admin" {
        let scholar_id = uploads::get_scholar_id_from_book(pool.get_ref(), book_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get scholar_id for book {}: {:?}", book_id, e);
                AppError {
                    message: Some("Book not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                }
            })?;

        let has_access =
            access::check_user_access_to_scholar(pool.get_ref(), auth.user_id, scholar_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check user access: {:?}", e);
                    AppError {
                        message: Some("Failed to verify permissions".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::InternalServerError,
                    }
                })?;

        if !has_access {
            return Err(AppError {
                message: Some(
                    "You don't have permission to upload to this scholar's content".to_string(),
                ),
                cause: None,
                error_type: AppErrorType::ForbiddenError,
            });
        }
    }

    // Create upload directory if it doesn't exist
    let upload_dir = &config.app_paths.uploads_dir;
    fs::create_dir_all(upload_dir).map_err(|e| {
        tracing::error!("Failed to create upload directory: {:?}", e);
        AppError {
            message: Some("Failed to prepare upload directory".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    let mut _description: Option<String> = None;
    let mut file_data: Option<(String, Vec<u8>, String)> = None;

    // Process multipart form data
    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {:?}", e);
        AppError {
            message: Some("Invalid file upload format".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::PayloadValidationError,
        }
    })? {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("");

        match field_name {
            "description" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| AppError {
                    message: Some("Failed to read description field".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::PayloadValidationError,
                })? {
                    data.extend_from_slice(&chunk);
                }
                let desc = String::from_utf8(data).map_err(|e| AppError {
                    message: Some("Invalid description encoding".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::PayloadValidationError,
                })?;
                if !desc.is_empty() {
                    _description = Some(desc);
                }
            }
            "file" => {
                let filename = content_disposition
                    .get_filename()
                    .ok_or_else(|| AppError {
                        message: Some("Filename is required".to_string()),
                        cause: None,
                        error_type: AppErrorType::PayloadValidationError,
                    })?
                    .to_string();

                // Validate MP3 file extension
                if !filename.to_lowercase().ends_with(".mp3") {
                    return Err(AppError {
                        message: Some("Only MP3 files are allowed".to_string()),
                        cause: None,
                        error_type: AppErrorType::PayloadValidationError,
                    });
                }

                let content_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .unwrap_or_else(|| "audio/mpeg".to_string());

                let mut file_bytes = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| AppError {
                    message: Some("Failed to read file data".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::PayloadValidationError,
                })? {
                    file_bytes.extend_from_slice(&chunk);
                    if file_bytes.len() > MAX_FILE_SIZE {
                        return Err(AppError {
                            message: Some("File size exceeds maximum limit (100MB)".to_string()),
                            cause: None,
                            error_type: AppErrorType::PayloadValidationError,
                        });
                    }
                }

                file_data = Some((filename, file_bytes, content_type));
            }
            _ => {
                // Skip unknown fields
                while let Some(_) = field.try_next().await.map_err(|_| AppError {
                    message: Some("Failed to skip unknown field".to_string()),
                    cause: None,
                    error_type: AppErrorType::PayloadValidationError,
                })? {}
            }
        }
    }

    let (original_filename, file_bytes, content_type) = file_data.ok_or_else(|| AppError {
        message: Some("File is required".to_string()),
        cause: None,
        error_type: AppErrorType::PayloadValidationError,
    })?;

    // Extract MP3 metadata (title and duration)
    let (title, duration) = extract_mp3_metadata(&file_bytes)?;

    tracing::info!(
        "Extracted MP3 metadata - Title: {}, Duration: {}",
        title,
        duration
    );

    // Generate unique filename
    let file_stem = Path::new(&original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Audio");

    let file_extension = Path::new(&original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("mp3");

    let random_id = Uuid::new_v4().to_string()[..5].to_string(); // 5 char random ID
    let unique_filename = format!("{}_{}.{}", file_stem, random_id, file_extension);
    let file_path = format!("{}/{}", upload_dir, unique_filename);

    fs::write(&file_path, &file_bytes).map_err(|e| {
        tracing::error!("Failed to write file {}: {:?}", file_path, e);
        AppError {
            message: Some("Failed to save file".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    // Save file metadata to database
    let upload_response = uploads::save_uploaded_file(
        pool.get_ref(),
        book_id, // Use extracted title from MP3
        &file_stem,
        &unique_filename,
        file_bytes.len() as i64,
        &content_type,
        &duration, // MP3 duration
        &random_id,
        auth.user_id,
    )
    .await
    .map_err(|e| {
        // Clean up file if database save fails
        let _ = fs::remove_file(&file_path);
        tracing::error!("Failed to save file metadata: {:?}", e);
        AppError {
            message: Some("Failed to save file metadata".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "File uploaded successfully".to_string(),
        data: Some(upload_response),
        pagination: None,
    }))
}

/// Track file download without actually downloading the file
///
/// This endpoint logs download activity for analytics purposes without
/// transferring the actual file. Useful for:
/// - Tracking downloads from external CDNs
/// - Analytics and metrics collection
/// - Monitoring user engagement
///
/// POST /api/v1/files/{file_id}/track-download
#[instrument(name = "Track Download", skip(pool, req, auth))]
#[post("/{file_id}/track-download")]
pub async fn track_download(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    file_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();

    // Verify file exists
    let file_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_files WHERE id = ? AND status = 'active'",
        file_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(AppError::db_error)?;

    if file_exists == 0 {
        return Err(AppError {
            message: Some("File not found".to_string()),
            cause: None,
            error_type: AppErrorType::NotFoundError,
        });
    }

    // Get user's active subscription (if any)
    let subscription_id =
        match subscriptions::get_user_active_subscription(&pool, auth.user_id).await {
            Ok(Some(subscription)) => Some(subscription.id),
            _ => None,
        };

    // Extract client IP and user agent
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|ip| ip.to_string());

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|ua| ua.to_string());

    // Log the download
    file_interactions::log_file_download(
        &pool,
        auth.user_id,
        subscription_id,
        file_id,
        client_ip,
        user_agent,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to log file download: {:?}", e);
        AppError {
            message: Some("Failed to track download".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    tracing::info!(
        "Download tracked for file {} by user {}",
        file_id,
        auth.user_id
    );

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Download tracked successfully".to_string(),
        data: Some(serde_json::json!({
            "file_id": file_id,
            "tracked_at": chrono::Utc::now()
        })),
        pagination: None,
    }))
}

/// Optimized file download using streaming (no memory loading)
///
/// Performance improvements:
/// - Uses NamedFile for zero-copy streaming directly from disk
/// - No memory allocation for file contents
/// - Supports range requests for partial downloads
/// - Efficient for large files (100MB+)
/// - Browser caching with Last-Modified headers
///
/// GET /api/v1/files/{file_id}/download
#[instrument(name = "Download File", skip(pool, config, auth))]
#[get("/{file_id}/download")]
pub async fn download_file(
    pool: web::Data<MySqlPool>,
    config: web::Data<crate::core::config::AppConfig>,
    auth: JwtMiddleware,
    file_id: web::Path<i32>,
) -> Result<NamedFile, AppError> {
    let file_id = file_id.into_inner();

    // Get file information (lightweight query)
    let file_info =
        uploads::get_file_download_info(pool.get_ref(), &config.app_paths.uploads_dir, file_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get file info: {:?}", e);
                AppError {
                    message: Some("File not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                }
            })?;

    // Open file using NamedFile for efficient streaming
    let named_file = NamedFile::open(&file_info.file_path)
        .map_err(|e| {
            tracing::error!("Failed to open file {}: {:?}", file_info.file_path, e);
            AppError {
                message: Some("File not found on disk".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::NotFoundError,
            }
        })?
        .use_last_modified(true)
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(
                file_info.filename.clone(),
            )],
        });

    tracing::info!("File {} streamed to user {}", file_id, auth.user_id);

    Ok(named_file)
}
