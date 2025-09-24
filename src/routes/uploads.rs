use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::TryStreamExt;
use sqlx::MySqlPool;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    core::{jwt_auth::JwtMiddleware, AppError, AppErrorType, AppSuccessResponse},
    db::{access, uploads, file_interactions, subscriptions},
};

const UPLOAD_DIR: &str = "./uploads";
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

#[instrument(name = "Upload File", skip(pool, payload))]
#[post("/books/{book_id}/upload")]
pub async fn upload_file(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    book_id: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();
    
    // Check if user has access to upload to this book's scholar
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

    let has_access = access::check_user_access_to_scholar(pool.get_ref(), auth.user_id, scholar_id)
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
            message: Some("You don't have permission to upload to this scholar's content".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    // Create upload directory if it doesn't exist
    fs::create_dir_all(UPLOAD_DIR).map_err(|e| {
        tracing::error!("Failed to create upload directory: {:?}", e);
        AppError {
            message: Some("Failed to prepare upload directory".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    let mut title = String::new();
    let mut description: Option<String> = None;
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
            "title" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    AppError {
                        message: Some("Failed to read title field".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::PayloadValidationError,
                    }
                })? {
                    data.extend_from_slice(&chunk);
                }
                title = String::from_utf8(data).map_err(|e| {
                    AppError {
                        message: Some("Invalid title encoding".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::PayloadValidationError,
                    }
                })?;
            }
            "description" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    AppError {
                        message: Some("Failed to read description field".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::PayloadValidationError,
                    }
                })? {
                    data.extend_from_slice(&chunk);
                }
                let desc = String::from_utf8(data).map_err(|e| {
                    AppError {
                        message: Some("Invalid description encoding".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::PayloadValidationError,
                    }
                })?;
                if !desc.is_empty() {
                    description = Some(desc);
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

                let content_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let mut file_bytes = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    AppError {
                        message: Some("Failed to read file data".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::PayloadValidationError,
                    }
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

    // Validate required fields
    if title.is_empty() {
        return Err(AppError {
            message: Some("Title is required".to_string()),
            cause: None,
            error_type: AppErrorType::PayloadValidationError,
        });
    }

    let (filename, file_bytes, content_type) = file_data.ok_or_else(|| AppError {
        message: Some("File is required".to_string()),
        cause: None,
        error_type: AppErrorType::PayloadValidationError,
    })?;

    // Generate unique filename
    let file_extension = Path::new(&filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let unique_filename = format!("{}_{}.{}", 
        Uuid::new_v4(), 
        chrono::Utc::now().timestamp(), 
        file_extension
    );
    let file_path = format!("{}/{}", UPLOAD_DIR, unique_filename);

    // Save file to disk
    let mut file = std::fs::File::create(&file_path).map_err(|e| {
        tracing::error!("Failed to create file {}: {:?}", file_path, e);
        AppError {
            message: Some("Failed to save file".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    file.write_all(&file_bytes).map_err(|e| {
        tracing::error!("Failed to write file data: {:?}", e);
        AppError {
            message: Some("Failed to save file".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    // Save file metadata to database
    let upload_response = uploads::save_uploaded_file(
        pool.get_ref(),
        book_id,
        &unique_filename,
        &file_path,
        file_bytes.len() as i64,
        &content_type,
        &title,
        description.as_deref(),
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

#[instrument(name = "Download File", skip(pool, req))]
#[get("/files/{file_id}/download")]
pub async fn download_file(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    file_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();

    // Check if user has access to download this file
    let has_access = uploads::check_file_access_permission(pool.get_ref(), auth.user_id, file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check file access: {:?}", e);
            AppError {
                message: Some("Failed to verify file permissions".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    if !has_access {
        return Err(AppError {
            message: Some("You don't have permission to download this file".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    // Get file information
    let file_info = uploads::get_file_download_info(pool.get_ref(), file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file info: {:?}", e);
            AppError {
                message: Some("File not found".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::NotFoundError,
            }
        })?;

    // Get user's active subscription (if any)
    let subscription_id = match subscriptions::get_user_active_subscription(&pool, auth.user_id).await {
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
    if let Err(e) = file_interactions::log_file_download(
        &pool,
        auth.user_id,
        subscription_id,
        file_id,
        client_ip,
        user_agent,
    ).await {
        tracing::warn!("Failed to log file download: {}", e);
        // Don't fail the download if logging fails
    }

    // Read file from disk
    let file_bytes = fs::read(&file_info.file_path).map_err(|e| {
        tracing::error!("Failed to read file {}: {:?}", file_info.file_path, e);
        AppError {
            message: Some("File not found on disk".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::NotFoundError,
        }
    })?;

    tracing::info!("File {} downloaded by user {}", file_id, auth.user_id);

    Ok(HttpResponse::Ok()
        .content_type(file_info.content_type.as_str())
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_info.filename),
        ))
        .insert_header(("Content-Length", file_info.file_size.to_string()))
        .body(file_bytes))
}