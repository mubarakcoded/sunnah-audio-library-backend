use crate::{
    core::{extract_user_id_from_request, jwt_auth::JwtMiddleware, slugify, AppConfig, AppError, AppErrorType, AppSuccessResponse},
    models::{pagination::{PaginationMeta, PaginationQuery}, scholars::{CreateScholarRequest, UpdateScholarRequest}},
};
use actix_multipart::Multipart;
use actix_web::{
    get, post, put,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};
use futures_util::TryStreamExt as _;
use std::fs;
use std::io::Write;
use uuid::Uuid;

use crate::db::scholars;
use sqlx::MySqlPool;
use tracing::instrument;

#[instrument(name = "Get Scholars", skip(pool))]
#[get("")]
pub async fn get_scholars(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = scholars::fetch_scholars(pool.get_ref(), &config, &pagination)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholars: {:?}", e);
            AppError {
                message: Some("Failed to fetch scholars".to_string()),
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
        message: "Scholars retrieved successfully".to_string(),
        data: Some(data),
        pagination: Some(pagination_meta),
    }))
}

#[instrument(name = "Get Scholars by State", skip(pool))]
#[get("/state/{state_id}")]
pub async fn get_scholars_by_state(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    state_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = scholars::fetch_scholars_by_state(
        pool.get_ref(),
        &config,
        state_id.into_inner(),
        &pagination,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch scholars by state: {:?}", e);
        AppError {
            message: Some("Failed to fetch scholars by state".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    let pagination_meta = PaginationMeta::new(pagination.page, pagination.per_page, total_items);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholars retrieved successfully".to_string(),
        data: Some(data),
        pagination: Some(pagination_meta),
    }))
}
#[instrument(name = "Get Scholar Details", skip(pool, config))]
#[get("/{scholar_id}")]
pub async fn get_scholar_details(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    scholar_id: web::Path<i32>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let scholar_id = scholar_id.into_inner();
    let user_id = extract_user_id_from_request(&req, &config);

    let scholar_details = scholars::get_scholar_details(pool.get_ref(), &config, scholar_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholar details: {:?}", e);
            match e.error_type {
                AppErrorType::NotFoundError => AppError {
                    message: Some("Scholar not found".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::NotFoundError,
                },
                _ => AppError {
                    message: Some("Failed to fetch scholar details".to_string()),
                    cause: Some(e.to_string()),
                    error_type: AppErrorType::InternalServerError,
                },
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholar details retrieved successfully".to_string(),
        data: Some(scholar_details),
        pagination: None,
    }))
}

#[instrument(name = "Get Scholar Statistics", skip(pool))]
#[get("/{scholar_id}/statistics")]
pub async fn get_scholar_statistics(
    pool: web::Data<MySqlPool>,
    scholar_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let scholar_id = scholar_id.into_inner();

    let statistics = scholars::get_scholar_statistics(pool.get_ref(), scholar_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholar statistics: {:?}", e);
            AppError {
                message: Some("Failed to fetch scholar statistics".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholar statistics retrieved successfully".to_string(),
        data: Some(statistics),
        pagination: None,
    }))
}
#[instrument(name = "Get Scholars Dropdown", skip(pool))]
#[get("/dropdown")]
pub async fn get_scholars_dropdown(
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, AppError> {
    let scholars = scholars::get_scholars_dropdown(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholars dropdown: {:?}", e);
            AppError {
                message: Some("Failed to fetch scholars dropdown".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholars dropdown retrieved successfully".to_string(),
        data: Some(scholars),
        pagination: None,
    }))
}

#[instrument(name = "Create Scholar", skip(pool, auth, payload))]
#[post("")]
pub async fn create_scholar(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    auth: JwtMiddleware,
    payload: Multipart,
) -> Result<impl Responder, AppError> {
    // Check if user is admin
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
        return Err(AppError {
            message: Some("Only admins can create scholars".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    // Parse multipart: fields (name, about, state_id) + optional image file
    let mut name: Option<String> = None;
    let mut about: Option<String> = None;
    let mut state_id: Option<i32> = None;
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
                let generated = format!("scholar_{}.{}", Uuid::new_v4(), file_ext);
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
            } else if field_name == "state_id" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid state_id: {}", e)))?.unwrap_or_default();
                state_id = String::from_utf8(bytes.to_vec()).ok().and_then(|s| s.parse::<i32>().ok());
            }
        }
    }

     let scholar_name = name.ok_or_else(|| AppError::bad_request("name is required"))?;
     let scholar_state_id = state_id.ok_or_else(|| AppError::bad_request("state_id is required"))?;
     let slug_value = slugify(&scholar_name);


    if let Some(existing_name) = scholars::check_duplicate_scholar(pool.get_ref(), &scholar_name, &slug_value).await? {
        return Err(AppError {
            message: Some(format!("A scholar with the name '{}' already exists", existing_name)),
            cause: None,
            error_type: AppErrorType::ConflictError,
        });
    }


    let request = CreateScholarRequest {
        name: scholar_name,
        about,
        state_id: scholar_state_id,
        image: image_filename,
    };

    let scholar_id = scholars::create_scholar(pool.get_ref(), &request, auth.user_id, &slug_value)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create scholar: {:?}", e);
            AppError {
                message: Some("Failed to create scholar".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        message: "Scholar created successfully".to_string(),
        data: Some(serde_json::json!({"id": scholar_id})),
        pagination: None,
    }))
}

#[instrument(name = "Update Scholar", skip(pool, auth, payload))]
#[put("/{scholar_id}")]
pub async fn update_scholar(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    auth: JwtMiddleware,
    scholar_id: web::Path<i32>,
    payload: Multipart,
) -> Result<impl Responder, AppError> {
    let scholar_id = scholar_id.into_inner();

    // Check if user is admin
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
        return Err(AppError {
            message: Some("Only admins can update scholars".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    // Parse multipart; same fields as create, all optional
    let mut name: Option<String> = None;
    let mut about: Option<String> = None;
    let mut state_id: Option<i32> = None;
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
                let generated = format!("scholar_{}.{}", Uuid::new_v4(), file_ext);
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
            } else if field_name == "state_id" {
                let bytes = field.try_next().await.map_err(|e| AppError::bad_request(format!("Invalid state_id: {}", e)))?.unwrap_or_default();
                state_id = String::from_utf8(bytes.to_vec()).ok().and_then(|s| s.parse::<i32>().ok());
            }
        }
    }

    let request = UpdateScholarRequest {
        name,
        about,
        state_id,
        image: image_filename,
    };

    scholars::update_scholar(pool.get_ref(), scholar_id, &request)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update scholar: {:?}", e);
            AppError {
                message: Some("Failed to update scholar".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholar updated successfully".to_string(),
        data: None::<()>,
        pagination: None,
    }))
}