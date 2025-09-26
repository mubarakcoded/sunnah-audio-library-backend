use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse, AppConfig, extract_user_id_from_request},
    models::pagination::{PaginationMeta, PaginationQuery},
};
use actix_web::{
    get,
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