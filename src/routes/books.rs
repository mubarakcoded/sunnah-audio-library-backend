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
