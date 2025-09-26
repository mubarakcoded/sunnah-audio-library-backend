use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse, AppConfig, extract_user_id_from_request, jwt_auth::JwtMiddleware},
    models::pagination::{PaginationMeta, PaginationQuery},
    models::scholars::{CreateScholarRequest, UpdateScholarRequest},
};
use actix_web::{
    get, post, put,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

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

#[instrument(name = "Create Scholar", skip(pool, auth))]
#[post("")]
pub async fn create_scholar(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    request: web::Json<CreateScholarRequest>,
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

    let scholar_id = scholars::create_scholar(pool.get_ref(), &request)
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

#[instrument(name = "Update Scholar", skip(pool, auth))]
#[put("/{scholar_id}")]
pub async fn update_scholar(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    scholar_id: web::Path<i32>,
    request: web::Json<UpdateScholarRequest>,
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