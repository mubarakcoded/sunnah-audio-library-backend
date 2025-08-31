use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse},
    models::pagination::{PaginationMeta, PaginationQuery},
};
use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};

use crate::db::scholars;
use sqlx::MySqlPool;
use tracing::instrument;

#[instrument(name = "Get Scholars", skip(pool))]
#[get("")]
pub async fn get_scholars(
    pool: web::Data<MySqlPool>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = scholars::fetch_scholars(pool.get_ref(), &pagination)
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
    state_id: web::Path<i32>,
    pagination: web::Query<PaginationQuery>,
) -> Result<impl Responder, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();

    let (data, total_items) = scholars::fetch_scholars_by_state(
        pool.get_ref(),
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
