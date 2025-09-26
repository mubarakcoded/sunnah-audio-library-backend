use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse, AppConfig, jwt_auth::JwtMiddleware},
    db::books,
    models::pagination::{PaginationMeta, PaginationQuery},
    models::books::{CreateBookRequest, UpdateBookRequest},
};
use actix_web::{
    get, post, put,
    web::{self},
    HttpResponse, Responder,
};

use sqlx::MySqlPool;
use tracing::instrument;

#[instrument(name = "Get Books by Scholar", skip(pool))]
#[get("/{scholar_id}/books")]
pub async fn get_books_by_scholar(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    scholar_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = books::fetch_books_by_scholar(
        pool.get_ref(),
        &config,
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
    config: web::Data<AppConfig>,
    book_id: web::Path<i32>,
    req: actix_web::HttpRequest,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();
    let user_id = crate::core::extract_user_id_from_request(&req, &config);

    let book_details = books::get_book_details(pool.get_ref(), &config, book_id, user_id)
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
}#[
instrument(name = "Get Books Dropdown", skip(pool))]
#[get("/dropdown")]
pub async fn get_books_dropdown(
    pool: web::Data<MySqlPool>,
    scholar_id: web::Query<Option<i32>>,
) -> Result<impl Responder, AppError> {
    let books = books::get_books_dropdown(pool.get_ref(), scholar_id.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch books dropdown: {:?}", e);
            AppError {
                message: Some("Failed to fetch books dropdown".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Books dropdown retrieved successfully".to_string(),
        data: Some(books),
        pagination: None,
    }))
}

#[instrument(name = "Create Book", skip(pool, auth))]
#[post("")]
pub async fn create_book(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    request: web::Json<CreateBookRequest>,
) -> Result<impl Responder, AppError> {
    // Check if user has access to create books for this scholar
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

    if user.role != "admin" {
        // Check if manager has access to this scholar
        let has_access = crate::db::access::check_user_access_to_scholar(
            pool.get_ref(), 
            auth.user_id, 
            request.scholar_id
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
                message: Some("You don't have permission to create books for this scholar".to_string()),
                cause: None,
                error_type: AppErrorType::ForbiddenError,
            });
        }
    }

    let book_id = books::create_book(pool.get_ref(), &request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create book: {:?}", e);
            AppError {
                message: Some("Failed to create book".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        message: "Book created successfully".to_string(),
        data: Some(serde_json::json!({"id": book_id})),
        pagination: None,
    }))
}

#[instrument(name = "Update Book", skip(pool, auth))]
#[put("/{book_id}")]
pub async fn update_book(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    book_id: web::Path<i32>,
    request: web::Json<UpdateBookRequest>,
) -> Result<impl Responder, AppError> {
    let book_id = book_id.into_inner();

    // Get current book to check scholar_id
    let current_book = sqlx::query!(
        "SELECT scholar_id FROM tbl_books WHERE id = ? AND status = 'active'",
        book_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to get book: {:?}", e);
        AppError {
            message: Some("Book not found".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::NotFoundError,
        }
    })?;

    // Check if user has access to update this book
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

    if user.role != "admin" {
        // Check if manager has access to this scholar
        let has_access = crate::db::access::check_user_access_to_scholar(
            pool.get_ref(), 
            auth.user_id, 
            current_book.scholar_id
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
                message: Some("You don't have permission to update this book".to_string()),
                cause: None,
                error_type: AppErrorType::ForbiddenError,
            });
        }

        // If changing scholar, check access to new scholar too
        if let Some(new_scholar_id) = request.scholar_id {
            if new_scholar_id != current_book.scholar_id {
                let has_new_access = crate::db::access::check_user_access_to_scholar(
                    pool.get_ref(), 
                    auth.user_id, 
                    new_scholar_id
                )
                .await
                .map_err(|e| {
                    tracing::error!("Failed to check user access to new scholar: {:?}", e);
                    AppError {
                        message: Some("Failed to verify permissions".to_string()),
                        cause: Some(e.to_string()),
                        error_type: AppErrorType::InternalServerError,
                    }
                })?;

                if !has_new_access {
                    return Err(AppError {
                        message: Some("You don't have permission to move this book to the specified scholar".to_string()),
                        cause: None,
                        error_type: AppErrorType::ForbiddenError,
                    });
                }
            }
        }
    }

    books::update_book(pool.get_ref(), book_id, &request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update book: {:?}", e);
            AppError {
                message: Some("Failed to update book".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Book updated successfully".to_string(),
        data: None::<()>,
        pagination: None,
    }))
}