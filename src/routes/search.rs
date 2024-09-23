use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use tracing::instrument;

use crate::{
    core::{jwt_auth, AppError, AppErrorType, AppSuccessResponse},
    db::{books, files, scholars},
};

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String, // search query
    pub page: Option<i64>,
    pub items_per_page: Option<i64>,
}

#[instrument(name = "Search Scholars, Books, Files", skip(pool, query))]
#[get("/search")]
pub async fn full_text_search(
    pool: web::Data<MySqlPool>,
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
    let items_per_page = query.items_per_page.unwrap_or(10);
    // Run searches concurrently

    let (scholars_res, books_res, files_res) = tokio::join!(
        scholars::search_scholars(pool.get_ref(), search_term, page, items_per_page),
        books::search_books(pool.get_ref(), search_term, page, items_per_page),
        files::search_files(pool.get_ref(), search_term, page, items_per_page),
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

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Search results retrieved successfully".to_string(),
        data: Some(serde_json::json!({
            "scholars": {
                "items": scholars.0,
                "pagination": {
                    "current_page": page,
                    "items_per_page": items_per_page,
                    "total_items": scholars.1,
                    "total_pages": (scholars.1 as f64 / items_per_page as f64).ceil() as i64
                }
            },
            "books": {
                "items": books.0,
                "pagination": {
                    "current_page": page,
                    "items_per_page": items_per_page,
                    "total_items": books.1,
                    "total_pages": (books.1 as f64 / items_per_page as f64).ceil() as i64
                }
            },
            "files": {
                "items": files.0,
                "pagination": {
                    "current_page": page,
                    "items_per_page": items_per_page,
                    "total_items": files.1,
                    "total_pages": (files.1 as f64 / items_per_page as f64).ceil() as i64
                }
            },
        })),
    }))
}