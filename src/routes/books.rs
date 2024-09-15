use crate::{
    core::{jwt_auth, AppError, AppErrorType, AppSuccessResponse},
    db::{books, states},
    models::{books::Book, scholars::Scholar},
};
use actix_web::{
    get, post,
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
) -> Result<impl Responder, AppError> {
    let result = books::fetch_books_by_scholar(pool.get_ref(), scholar_id.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch books: {:?}", e);
            AppError {
                message: Some("Failed to fetch books".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Books retrieved successfully".to_string(),
        data: Some(result),
    }))
}
