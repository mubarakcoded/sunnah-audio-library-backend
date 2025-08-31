use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use sqlx::MySqlPool;
use tracing::instrument;

// const BANK_CODE: &str = "SHUNKU";
// const BANK_NAME: &str = "009291";

use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse},
    db::files,
    models::{
        files::RelatedFilesParams,
        pagination::{PaginationMeta, PaginationQuery},
    },
};

#[instrument(name = "Get Files by Book", skip(pool))]
#[get("/{book_id}/files")]
pub async fn get_files_by_book(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) =
        files::fetch_files_by_book(pool.get_ref(), book_id.into_inner(), &pagination)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch files by book: {:?}", e);
                AppError {
                    message: Some("Failed to fetch files".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::InternalServerError,
                }
            })?;

    let pagination_meta =
        PaginationMeta::new(pagination.page, pagination.per_page, total_items);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Files retrieved successfully".to_string(),
        data: Some(data),
        pagination: Some(pagination_meta),
    }))
}

#[instrument(name = "Get Recent Files", skip(pool, pagination))]
#[get("/recent")]
pub async fn get_recent_files(
    pool: web::Data<MySqlPool>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = files::fetch_recent_files(pool.get_ref(), &pagination)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch recent files: {:?}", e);
            AppError {
                message: Some("Failed to fetch recent files".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    let pagination_meta = PaginationMeta::new(pagination.page, pagination.per_page, total_items);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Recent files retrieved successfully".to_string(),
        data: Some(data),
        pagination: Some(pagination_meta),
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
        pagination: None,
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

    let pagination_meta = PaginationMeta::new(page, items_per_page, total_count);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Related files retrieved successfully".to_string(),
        data: Some(related_files),
        pagination: Some(pagination_meta),
    }))
}
