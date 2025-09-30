use actix_web::{
    get, put,
    web::{self},
    HttpResponse, Responder, HttpRequest,
};
use sqlx::MySqlPool;
use tracing::instrument;

use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse, AppConfig, extract_user_id_from_request, jwt_auth::JwtMiddleware},
    db::files,
    models::pagination::{PaginationMeta, PaginationQuery},
    models::files::UpdateFileRequest,
};

#[instrument(name = "Get Files by Book", skip(pool, config))]
#[get("/{book_id}/files")]
pub async fn get_files_by_book(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    book_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let user_id = extract_user_id_from_request(&req, &config);

    let (data, total_items) = files::fetch_files_by_book_with_stats(
        pool.get_ref(),
        &config,
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

#[instrument(name = "Get Recent Files", skip(pool, config, pagination))]
#[get("/recent")]
pub async fn get_recent_files(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    pagination: web::Query<PaginationQuery>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let user_id = extract_user_id_from_request(&req, &config);

    let (data, total_items) =
        files::fetch_recent_files_with_stats(pool.get_ref(), &config, &pagination, user_id)
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
#[instrument(name = "Get All Files for Play All", skip(pool, config))]
#[get("/{book_id}/play-all")]
pub async fn get_all_files_for_play_all(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    book_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    let play_all_data = files::get_all_files_for_book_play_all(pool.get_ref(), &config, book_id)
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
}#
[instrument(name = "Update File", skip(pool, auth))]
#[put("/{file_id}")]
pub async fn update_file(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    file_id: web::Path<i32>,
    request: web::Json<UpdateFileRequest>,
) -> Result<impl Responder, AppError> {
    let file_id = file_id.into_inner();

    let user = crate::db::users::get_user_by_id(pool.get_ref(), auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            AppError {
                message: Some("User not found".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::NotFoundError,
            }
        })?;

    // if user.role != "admin" {
    //     let can_update = files::check_file_owner(pool.get_ref(), auth.user_id, file_id)
    //         .await
    //         .map_err(|e| {
    //             tracing::error!("Failed to check file permissions: {:?}", e);
    //             AppError {
    //                 message: Some("Failed to verify permissions".to_string()),
    //                 cause: Some(e.to_string()),
    //                 error_type: AppErrorType::InternalServerError,
    //             }
    //         })?;

    //     if !can_update {
    //         return Err(AppError {
    //             message: Some("You don't have permission to update this file".to_string()),
    //             cause: None,
    //             error_type: AppErrorType::ForbiddenError,
    //         });
    //     }
    // }

 

    // If changing book, check if user has access to the new book's scholar
    if let Some(new_book_id) = request.book_id {
        if user.role != "admin" {
            // Get the scholar_id for the new book
            let scholar_id = sqlx::query_scalar!(
                "SELECT scholar_id FROM tbl_books WHERE id = ? AND status = 'active'",
                new_book_id
            )
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to get book scholar: {:?}", e);
                AppError {
                    message: Some("Book not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                }
            })?;

            let has_access = crate::db::access::check_user_access_to_scholar(
                pool.get_ref(), 
                auth.user_id, 
                scholar_id
            )
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
                    message: Some("You don't have permission to move this file to the specified book".to_string()),
                    cause: None,
                    error_type: AppErrorType::ForbiddenError,
                });
            }
        }
    }

    // If changing scholar directly, check permissions
    if let Some(new_scholar_id) = request.scholar_id {
        if user.role != "admin" {
            let has_access = crate::db::access::check_user_access_to_scholar(
                pool.get_ref(),
                auth.user_id,
                new_scholar_id
            )
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
                    message: Some("You don't have permission to assign this file to the specified scholar".to_string()),
                    cause: None,
                    error_type: AppErrorType::ForbiddenError,
                });
            }
        }
    }

    files::update_file(pool.get_ref(), file_id, &request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update file: {:?}", e);
            AppError {
                message: Some("Failed to update file".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "File updated successfully".to_string(),
        data: None::<()>,
        pagination: None,
    }))
}