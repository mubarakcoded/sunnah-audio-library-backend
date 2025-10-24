use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use tracing::instrument;

use crate::{
    core::{AppConfig, AppError, AppErrorType, AppSuccessResponse},
    db::{books, files, scholars},
    models::pagination::PaginationMeta,
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String, // search query
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[instrument(name = "Search Scholars, Books, Files", skip(pool, query))]
#[get("/search")]
pub async fn full_text_search(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    query: web::Query<SearchParams>,
) -> Result<impl Responder, AppError> {
    let search_term = query.q.trim();
    if search_term.is_empty() {
        return Err(AppError {
            message: Some("Search query cannot be empty".to_string()),
            cause: None,
            error_type: AppErrorType::PayloadValidationError,
        });
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(30);
    // Run searches concurrently

    let (scholars_res, books_res, files_res) = tokio::join!(
        scholars::search_scholars(pool.get_ref(), &config, search_term, page, per_page),
        books::search_books(pool.get_ref(), &config, search_term, page, per_page),
        files::search_files(pool.get_ref(), &config, search_term, page, per_page),
    );

    let (scholars, books, files) = (
        scholars_res.map_err(|e| {
            tracing::error!("Failed to search scholars: {:?}", e);
            AppError {
                message: Some("Failed to search scholars".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?,
        books_res.map_err(|e| {
            tracing::error!("Failed to search books: {:?}", e);
            AppError {
                message: Some("Failed to search books".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?,
        files_res.map_err(|e| {
            tracing::error!("Failed to search files: {:?}", e);
            AppError {
                message: Some("Failed to search files".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?,
    );

    let scholars_pagination = PaginationMeta::new(page, per_page, scholars.1);
    let books_pagination = PaginationMeta::new(page, per_page, books.1);
    let files_pagination = PaginationMeta::new(page, per_page, files.1);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Search results retrieved successfully".to_string(),
        data: Some(serde_json::json!({
            "scholars": {
                "items": scholars.0,
                "pagination": scholars_pagination
            },
            "books": {
                "items": books.0,
                "pagination": books_pagination
            },
            "files": {
                "items": files.0,
                "pagination": files_pagination
            },
        })),
        pagination: None,
    }))
}
