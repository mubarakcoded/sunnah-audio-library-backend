use sqlx::PgPool;
use tracing;
use uuid::Uuid;

use crate::core::{AppError, AppErrorType};
use crate::models::cable_networks::CablePackages;
use crate::models::cable_networks::CableProviders;

pub async fn get_cable_providers(db_pool: &PgPool) -> Result<Vec<CableProviders>, AppError> {
    let cable_providers = sqlx::query_as!(
        CableProviders,
        "SELECT provider_id, provider_name, logo_url, slug, status
         FROM cable_providers
         WHERE status = 'active'"
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(cable_providers)
}

pub async fn get_cable_packages_by_provider(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<Vec<CablePackages>, AppError> {
    let cable_packages = sqlx::query_as!(
        CablePackages,
        "SELECT package_id, package_name, service_code, package_description, package_price
         FROM cable_packages
         WHERE provider_id = $1 AND status ='active'",
        provider_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(cable_packages)
}

pub async fn get_network_info(
    db_pool: &PgPool,
    entity_id: &Uuid,
    provider_name: &str,
) -> Result<(String, String), AppError> {
    let query = format!(
        "SELECT provider_name AS network_name, variation_codes->>$1 AS variation_code FROM cable_providers WHERE provider_id = $2"
    );

    let result = sqlx::query_as::<_, (String, String)>(&query)
        .bind(provider_name)
        .bind(entity_id)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok((network_name, variation_code)) => Ok((network_name, variation_code)),
        Err(_) => Err(AppError {
            message: Some("Provider information not found".to_string()),
            cause: None,
            error_type: AppErrorType::NotFoundError,
        }),
    }
}

pub async fn get_plan_info(
    db_pool: &PgPool,
    network_id: &Uuid,
    plan_id: &Uuid,
    provider_name: &str,
) -> Result<(String, String), AppError> {
    let query = format!(
        "SELECT package_name, variation_codes->>$1 AS variation_code 
         FROM cable_packages 
         WHERE provider_id = $2 AND package_id = $3"
    );

    let result = sqlx::query_as::<_, (String, String)>(&query)
        .bind(provider_name)
        .bind(network_id)
        .bind(plan_id)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok((plan_name, variation_code)) => Ok((plan_name, variation_code)),
        Err(_) => Err(AppError {
            message: Some("Cable package information not found".to_string()),
            cause: None,
            error_type: AppErrorType::NotFoundError,
        }),
    }
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    tracing::error!("Database error: {:?}", err);
    AppError {
        message: Some(format!("Database error: {}", err)),
        cause: Some(err.to_string()),
        error_type: AppErrorType::DbError,
    }
}
