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
    models::common::PaginationParams,
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
#[get("/files/explore")]
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
