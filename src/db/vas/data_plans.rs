use sqlx::PgPool;
use tracing;
use uuid::Uuid;

use crate::core::{AppError, AppErrorType};
use crate::models::data_plans::DataPlans;

pub async fn fetch_data_plans_by_provider_id(
    db_pool: &PgPool,
    network_id: &Uuid,
) -> Result<Vec<DataPlans>, AppError> {
    let data_plan = sqlx::query_as!(
        DataPlans,
        "SELECT plan_id, plan_name, data_amount, data_unit, validity_period, validity_unit, price
         FROM data_plans
         WHERE network_id = $1",
        network_id
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(data_plan)
}

pub async fn fetch_data_plans_by_service_code(
    db_pool: &PgPool,
    service_code: &str,
) -> Result<Vec<DataPlans>, AppError> {
    let data_plans = sqlx::query_as!(
        DataPlans,
        "SELECT dp.plan_id, dp.plan_name, dp.data_amount, dp.data_unit, dp.validity_period, dp.validity_unit, dp.price
         FROM data_plans dp
         JOIN mobile_networks mn ON dp.network_id = mn.network_id
         WHERE mn.service_code = $1",
         service_code
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(data_plans)
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    tracing::error!("Database error: {:?}", err);
    AppError {
        message: Some(format!("Database error: {}", err)),
        cause: Some(err.to_string()),
        error_type: AppErrorType::DbError,
    }
}
