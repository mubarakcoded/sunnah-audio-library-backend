use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse},
    db::books,
    models::pagination::{PaginationMeta, PaginationQuery},
};
use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};

use sqlx::MySqlPool;
use tracing::instrument;

#[instrument(name = "Get Books by Scholar", skip(pool))]
#[get("/{scholar_id}/books")]
pub async fn get_books_by_scholar(
    pool: web::Data<MySqlPool>,
    scholar_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = books::fetch_books_by_scholar(
        pool.get_ref(),
        scholar_id.into_inner(),
        &pagination,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch books: {:?}", e);
        AppError {
            message: Some("Failed to fetch books".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    let pagination_meta = PaginationMeta::new(
        pagination.page,
        pagination.per_page,
        total_items,
    );

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Books retrieved successfully".to_string(),
        data: Some(data),
        pagination: Some(pagination_meta),
    }))
}
#[instrument(name = "Get Book Details", skip(pool))]
#[get("/{book_id}")]
pub async fn get_book_details(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    let book_details = books::get_book_details(pool.get_ref(), book_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch book details: {:?}", e);
            match e.error_type {
                AppErrorType::NotFoundError => AppError {
                    message: Some("Book not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                },
                _ => AppError {
                    message: Some("Failed to fetch book details".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::InternalServerError,
                }
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Book details retrieved successfully".to_string(),
        data: Some(book_details),
        pagination: None,
    }))
}

#[instrument(name = "Get Book Statistics", skip(pool))]
#[get("/{book_id}/statistics")]
pub async fn get_book_statistics(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    let statistics = books::get_book_statistics(pool.get_ref(), book_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch book statistics: {:?}", e);
            AppError {
                message: Some("Failed to fetch book statistics".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Book statistics retrieved successfully".to_string(),
        data: Some(statistics),
        pagination: None,
    }))
}