use sqlx::PgPool;
use tracing;
use uuid::Uuid;

use crate::core::{AppError, AppErrorType};
use crate::models::electricity_discos::ElectricityDisco;

pub async fn list_electricity_discos(pool: &PgPool) -> Result<Vec<ElectricityDisco>, AppError> {
    let discos = sqlx::query_as!(
        ElectricityDisco,
        "SELECT disco_id, disco_code AS disco_name, logo_url, slug FROM electricity_discos
        WHERE status = 'active' ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await;

    match discos {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            Err(AppError::db_error(e))
        }
    }
}

pub async fn get_disco_info(
    db_pool: &PgPool,
    disco_id: &Uuid,
    meter_type: &String,
    provider_name: &String,
) -> Result<(String, String), AppError> {
    let query = format!(
        "SELECT disco_code AS disco_name, variation_codes->$1->>$2 AS variation_code FROM electricity_discos WHERE disco_id = $3"
    );

    let result = sqlx::query_as::<_, (String, String)>(&query)
        .bind(meter_type)
        .bind(provider_name)
        .bind(disco_id)
        .fetch_one(db_pool)
        .await;

    match result {
        Ok((disco_name, variation_code)) => Ok((disco_name, variation_code)),
        Err(e) => Err(AppError {
            message: Some(e.to_string()),
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
