use crate::{
    core::{jwt_auth::JwtMiddleware, slugify, AppConfig, AppError, AppErrorType, AppSuccessResponse},
    db::books,
    models::{books::{CreateBookRequest, UpdateBookRequest}, pagination::{PaginationMeta, PaginationQuery}},
};
use actix_multipart::Multipart;
use actix_web::{
    get, post, put,
    web::{self},
    HttpResponse, Responder,
};
use futures_util::TryStreamExt as _;
use std::fs;
use std::io::Write;
use uuid::Uuid;

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

#[instrument(name = "Create Book", skip(pool, auth, payload))]
#[post("")]
pub async fn create_book(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    auth: JwtMiddleware,
    payload: Multipart,
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

    // Permission check occurs after parsing multipart when scholar_id is known

    // Parse multipart fields
    let mut name: Option<String> = None;
    let mut about: Option<String> = None;
    let mut scholar_id_field: Option<i32> = None;
    let mut image_field_data: Option<Vec<u8>> = None;
    let mut image_extension: Option<String> = None;

    let images_dir = &config.app_paths.images_dir;
    fs::create_dir_all(images_dir).ok();

    let mut payload = payload;
    while let Some(mut field) = payload.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid multipart: {}", e)))? {
        let cd = field.content_disposition();
        let field_name = cd.get_name().unwrap_or("").to_string();
        
        if !field_name.is_empty() {
            if field_name == "image" {
                // Store image data in memory, don't write to disk yet
                let file_ext = cd.get_filename()
                    .and_then(|f| std::path::Path::new(f).extension().and_then(|e| e.to_str()))
                    .unwrap_or("jpg")
                    .to_string();
                image_extension = Some(file_ext);
                
                let mut img_data = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| AppError::internal_error(format!("Failed to read image: {}", e)))? {
                    img_data.extend_from_slice(&chunk);
                }
                image_field_data = Some(img_data);
            } else if field_name == "name" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid name: {}", e)))?.unwrap_or_default();
                name = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            } else if field_name == "about" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid about: {}", e)))?.unwrap_or_default();
                about = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            } else if field_name == "scholar_id" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid scholar_id: {}", e)))?.unwrap_or_default();
                scholar_id_field = String::from_utf8(bytes.to_vec()).ok().and_then(|s| s.parse::<i32>().ok());
            }
        }
    }

    let book_name = name.ok_or_else(|| AppError::bad_request("name is required"))?;
    let book_scholar_id = scholar_id_field.ok_or_else(|| AppError::bad_request("scholar_id is required"))?;
    let slug_value = slugify(&book_name);

    // After parsing, validate permission with actual scholar_id
    if user.role != "admin" {
        let sid = scholar_id_field.ok_or_else(|| AppError::bad_request("scholar_id is required"))?;
        let has_access = crate::db::access::check_user_access_to_scholar(
            pool.get_ref(), auth.user_id, sid
        )
        .await
        .map_err(|e| AppError::internal_error(format!("Failed to verify permissions: {}", e)))?;
        if !has_access { return Err(AppError::forbidden_error("You don't have permission to create books for this scholar")); }
    }

    if let Some(existing_name) = books::check_duplicate_book(pool.get_ref(), &book_name, book_scholar_id, &slug_value).await? {
        return Err(AppError {
            message: Some(format!("A book with the name '{}' already exists for this scholar", existing_name)),
            cause: None,
            error_type: AppErrorType::ConflictError,
        });
    }

    // Now process and save the image if it exists
    let mut image_filename: Option<String> = None;
    if let Some(img_data) = image_field_data {
        let images_dir = &config.app_paths.images_dir;
        fs::create_dir_all(images_dir).ok();
        
        let file_ext = image_extension.unwrap_or_else(|| "jpg".to_string());
        let generated = format!("book_{}.{}", Uuid::new_v4(), file_ext);
        let filepath = format!("{}/{}", images_dir, generated);
        
        fs::write(&filepath, img_data)
            .map_err(|e| AppError::internal_error(format!("Failed to save image: {}", e)))?;
        
        image_filename = Some(generated);
    }

    let request = CreateBookRequest {
        name: book_name,
        about,
        scholar_id: book_scholar_id,
        image: image_filename,
    };

    let book_id = books::create_book(pool.get_ref(), &request, &slug_value, auth.user_id)
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

#[instrument(name = "Update Book", skip(pool, auth, payload))]
#[put("/{book_id}")]
pub async fn update_book(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    auth: JwtMiddleware,
    book_id: web::Path<i32>,
    payload: Multipart,
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

    // Permission checks will occur after parsing potential new scholar_id

    // Parse multipart changes
    let mut name: Option<String> = None;
    let mut about: Option<String> = None;
    let mut scholar_id: Option<i32> = None;
    let mut image_filename: Option<String> = None;

    let images_dir = &config.app_paths.images_dir;
    fs::create_dir_all(images_dir).ok();

    let mut payload = payload;
    while let Some(mut field) = payload.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid multipart: {}", e)))? {
        let cd = field.content_disposition();
        let field_name = cd.get_name().unwrap_or("").to_string();
        if !field_name.is_empty() {
            if field_name == "image" {
                let file_ext = cd.get_filename().and_then(|f| std::path::Path::new(f).extension().and_then(|e| e.to_str())).unwrap_or("jpg");
                let generated = format!("book_{}.{}", Uuid::new_v4(), file_ext);
                let filepath = format!("{}/{}", images_dir, generated);
                let mut f = fs::File::create(&filepath)
                    .map_err(|e| AppError::internal_error(format!("Failed to create image: {}", e)))?;
                while let Some(chunk) = field.try_next().await.map_err(|e| AppError::internal_error(format!("Failed to read image: {}", e)))? {
                    f.write_all(&chunk).map_err(|e| AppError::internal_error(format!("Failed to write image: {}", e)))?;
                }
                image_filename = Some(generated);
            } else if field_name == "name" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid name: {}", e)))?.unwrap_or_default();
                name = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            } else if field_name == "about" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid about: {}", e)))?.unwrap_or_default();
                about = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            } else if field_name == "scholar_id" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid scholar_id: {}", e)))?.unwrap_or_default();
                scholar_id = String::from_utf8(bytes.to_vec()).ok().and_then(|s| s.parse::<i32>().ok());
            }
        }
    }

    // Run permission checks now that potential new scholar_id is known
    if user.role != "admin" {
        // Must have access to current scholar
        let has_access = crate::db::access::check_user_access_to_scholar(
            pool.get_ref(), auth.user_id, current_book.scholar_id
        )
        .await
        .map_err(|e| AppError::internal_error(format!("Failed to verify permissions: {}", e)))?;
        if !has_access {
            return Err(AppError::forbidden_error("You don't have permission to update this book"));
        }

        // If moving to a different scholar, must have access there too
        if let Some(new_scholar_id) = scholar_id {
            if new_scholar_id != current_book.scholar_id {
                let has_new_access = crate::db::access::check_user_access_to_scholar(
                    pool.get_ref(), auth.user_id, new_scholar_id
                )
                .await
                .map_err(|e| AppError::internal_error(format!("Failed to verify permissions: {}", e)))?;
                if !has_new_access {
                    return Err(AppError::forbidden_error("You don't have permission to move this book to the specified scholar"));
                }
            }
        }
    }

    let request = UpdateBookRequest { name, about, scholar_id, image: image_filename };

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