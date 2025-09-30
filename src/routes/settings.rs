use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use tracing::instrument;

use crate::core::{AppError, AppErrorType, AppSuccessResponse};
use crate::db::settings::fetch_site_settings;

#[instrument(name = "Get Site Settings", skip(pool))]
#[get("/settings")]
pub async fn get_site_settings(
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, AppError> {
    let settings = fetch_site_settings(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch site settings: {:?}", e);
            AppError {
                message: Some("Failed to fetch site settings".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Site settings retrieved successfully".to_string(),
        data: Some(settings),
        pagination: None,
    }))
}


