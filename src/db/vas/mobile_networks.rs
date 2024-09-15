use sqlx::PgPool;
use tracing;
use uuid::Uuid;

use crate::core::{AppError, AppErrorType};
use crate::models::mobile_networks::MobileNetwork;

pub async fn get_all_mobile_networks(pool: &PgPool) -> Result<Vec<MobileNetwork>, AppError> {
    let mobile_networks = sqlx::query_as!(
        MobileNetwork,
        "SELECT network_id, network_name, logo_url, service_code, slug, supports_airtime, supports_data, status
         FROM mobile_networks
         WHERE status = 'active'"
    )
    .fetch_all(pool)
    .await;

    match mobile_networks {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            Err(AppError::db_error(e))
        }
    }
}

pub async fn get_mobile_network_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<MobileNetwork>, AppError> {
    let mobile_network = sqlx::query_as!(
        MobileNetwork,
        "SELECT network_id, network_name, logo_url, service_code, slug, supports_airtime, supports_data, status
         FROM mobile_networks
         WHERE network_id = $1",
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(mobile_network)
}

pub async fn create_mobile_network(
    pool: &PgPool,
    network: &MobileNetwork,
) -> Result<MobileNetwork, AppError> {
    let result = sqlx::query_as!(
        MobileNetwork,
        "INSERT INTO mobile_networks (network_name, logo_url, service_code, slug, supports_airtime, supports_data, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING network_id, network_name, logo_url, service_code, slug, supports_airtime, supports_data, status",
        network.network_name,
        network.logo_url,
        network.service_code,
        network.slug,
        network.supports_airtime,
        network.supports_data,
        network.status
    )
    .fetch_one(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(result)
}

pub async fn update_mobile_network(
    pool: &PgPool,
    id: Uuid,
    network: &MobileNetwork,
) -> Result<MobileNetwork, AppError> {
    let result = sqlx::query_as!(
        MobileNetwork,
        "UPDATE mobile_networks
         SET network_name = $1, logo_url = $2, service_code = $3, slug = $4, supports_airtime = $5, supports_data = $6, status = $7
         WHERE network_id = $8
         RETURNING network_id, network_name, logo_url, service_code, slug, supports_airtime, supports_data, status",
        network.network_name,
        network.logo_url,
        network.service_code,
        network.slug,
        network.supports_airtime,
        network.supports_data,
        network.status,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(result)
}

pub async fn delete_mobile_network(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "DELETE FROM mobile_networks
         WHERE network_id = $1",
        id
    )
    .execute(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_network_info(
    db_pool: &PgPool,
    column_name: &str,
    entity_id: &Uuid,
    provider_name: &str,
) -> Result<(String, String), AppError> {
    let query = format!(
        "SELECT network_name, {}->>$1 AS variation_code FROM mobile_networks WHERE network_id = $2",
        column_name
    );

    let result = sqlx::query_as::<_, (String, String)>(&query)
        .bind(provider_name)
        .bind(entity_id)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok((network_name, variation_code)) => Ok((network_name, variation_code)),
        Err(_) => Err(AppError {
            message: Some("Network information not found".to_string()),
            cause: None,
            error_type: AppErrorType::NotFoundError,
        }),
    }
}

pub async fn get_data_plan_info(
    db_pool: &PgPool,
    column_name: &str,
    network_id: &Uuid,
    data_plan_id: &Uuid,
    provider_name: &str,
) -> Result<(String, String), AppError> {
    let query = format!(
        "SELECT plan_name, {}->>$1 AS variation_code 
         FROM data_plans 
         WHERE network_id = $2 AND plan_id = $3",
        column_name
    );

    let result = sqlx::query_as::<_, (String, String)>(&query)
        .bind(provider_name)
        .bind(network_id)
        .bind(data_plan_id)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok((plan_name, variation_code)) => Ok((plan_name, variation_code)),
        Err(_) => Err(AppError {
            message: Some("Data plan information not found".to_string()),
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
