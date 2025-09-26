use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::MySqlPool;
use tracing::instrument;

// Helper function to extract user ID from optional JWT token
fn extract_user_id_from_request(req: &HttpRequest) -> Option<i32> {
    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })?;

    let claims = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret("UDAFMIEOLANAOIEWOLADFWEALMOPNVALKAE".as_ref()),
        &Validation::default(),
    )
    .ok()?
    .claims;

    claims.sub.parse().ok()
}

use crate::{
    core::{jwt_auth::JwtClaims, AppError, AppErrorType, AppSuccessResponse},
    db::files,
    models::pagination::{PaginationMeta, PaginationQuery},
};
use actix_web::HttpRequest;

#[instrument(name = "Get Files by Book", skip(pool))]
#[get("/{book_id}/files")]
pub async fn get_files_by_book(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let user_id = extract_user_id_from_request(&req);

    let (data, total_items) = files::fetch_files_by_book_with_stats(
        pool.get_ref(),
        book_id.into_inner(),
        &pagination,
        user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch files by book: {:?}", e);
        AppError {
            message: Some("Failed to fetch files".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    let pagination_meta = PaginationMeta::new(pagination.page, pagination.per_page, total_items);

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
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let user_id = extract_user_id_from_request(&req);

    let (data, total_items) =
        files::fetch_recent_files_with_stats(pool.get_ref(), &pagination, user_id)
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

#[instrument(name = "Get Related Files", skip(pool, pagination))]
#[get("/{file_id}/related")]
pub async fn get_related_files(
    pool: web::Data<MySqlPool>,
    file_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let file_id = file_id.into_inner();

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
        files::fetch_related_files(pool.get_ref(), book_id, file_id, &pagination).await?;

    let pagination_meta = PaginationMeta::new(pagination.page, pagination.per_page, total_count);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Related files retrieved successfully".to_string(),
        data: Some(related_files),
        pagination: Some(pagination_meta),
    }))
}
#[instrument(name = "Get All Files for Play All", skip(pool))]
#[get("/{book_id}/play-all")]
pub async fn get_all_files_for_play_all(
    pool: web::Data<MySqlPool>,
    book_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    let play_all_data = files::get_all_files_for_book_play_all(pool.get_ref(), book_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch files for play all: {:?}", e);
            match e.error_type {
                AppErrorType::NotFoundError => AppError {
                    message: Some("Book not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                },
                _ => AppError {
                    message: Some("Failed to fetch files for play all".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::InternalServerError,
                }
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Play all files retrieved successfully".to_string(),
        data: Some(play_all_data),
        pagination: None,
    }))
}