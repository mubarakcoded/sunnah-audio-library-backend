use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use bigdecimal::{BigDecimal, Zero};
// use chrono::Utc;
use sqlx::{MySqlPool, PgPool};
use tracing::instrument;
use uuid::Uuid;

// const BANK_CODE: &str = "SHUNKU";
// const BANK_NAME: &str = "009291";

use crate::{
    core::{jwt_auth, AppError, AppErrorType, AppSuccessResponse},
    db::{files, states},
    models::{common::PaginationParams, files::RelatedFilesParams},
};

#[instrument(name = "Get Files by Book", skip(pool))]
#[get("/{book_id}/files")]
pub async fn get_files_by_book(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let result = files::fetch_files_by_book(pool.get_ref(), book_id.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch files: {:?}", e);
            AppError {
                message: Some("Failed to fetch files".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Files retrieved successfully".to_string(),
        data: Some(result),
    }))
}

#[instrument(name = "Get Recent Files", skip(pool, query))]
#[get("/explore")]
pub async fn get_recent_files(
    pool: web::Data<MySqlPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, AppError> {
    // Default values for pagination if not provided
    let page = query.page.unwrap_or(1);
    let items_per_page = query.items_per_page.unwrap_or(15);

    let (result, total_count) = files::fetch_recent_files(pool.get_ref(), page, items_per_page)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch recent files: {:?}", e);
            AppError {
                message: Some("Failed to fetch recent files".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Recent files retrieved successfully".to_string(),
        data: Some(serde_json::json!({
            "files": result,
            "pagination": {
                "current_page": page,
                "items_per_page": items_per_page,
                "total_items": total_count,
                "total_pages": (total_count as f64 / items_per_page as f64).ceil() as i64
            }
        })),
    }))
}

#[instrument(name = "View File", skip(pool))]
#[get("/{file_id}/view")]
pub async fn view_file(
    pool: web::Data<MySqlPool>,
    file_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();
    let file_details = files::fetch_file_details(pool.get_ref(), file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch file details {}: {:?}", file_id, e);
            AppError {
                message: Some("Failed to fetch file details".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "File details retrieved successfully".to_string(),
        data: Some(file_details),
    }))
}

#[instrument(name = "Get Related Files", skip(pool, query))]
#[get("/{file_id}/related")]
pub async fn get_related_files(
    pool: web::Data<MySqlPool>,
    file_id: web::Path<i32>,
    query: web::Query<RelatedFilesParams>,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();
    let page = query.page.unwrap_or(1);
    let items_per_page = query.items_per_page.unwrap_or(10);

    // First, fetch the book_id of the current file
    let book_id = files::fetch_book_id_for_file(pool.get_ref(), file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch book_id for file {}: {:?}", file_id, e);
            AppError {
                message: Some("Failed to fetch book_id for file".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    // Fetch related files
    let (related_files, total_count) =
        files::fetch_related_files(pool.get_ref(), book_id, file_id, page, items_per_page).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Related files retrieved successfully".to_string(),
        data: Some(serde_json::json!({
            "files": related_files,
            "pagination": {
                "current_page": page,
                "items_per_page": items_per_page,
                "total_items": total_count,
                "total_pages": (total_count as f64 / items_per_page as f64).ceil() as i64
            }
        })),
    }))
}
