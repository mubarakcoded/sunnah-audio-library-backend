use crate::core::{AppError, AppErrorType};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct CurrenciesTbl {
    pub currency_id: Uuid,
    pub currency_code: String,
    pub currency_name: String,
    pub currency_type: String,
    pub currency_symbol: String,
    pub status: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl CurrenciesTbl {

    #[tracing::instrument(name = "Getting Currency details by currency_code", skip(db_pool))]
    pub async fn find_by_currency_code(db_pool: &PgPool, currency_code: String) -> Result<CurrenciesTbl, AppError> {
        match sqlx::query_as::<_, CurrenciesTbl>("SELECT * FROM currencies WHERE currency_code = $1")
            .bind(&currency_code)
            .fetch_optional(db_pool)
            .await
        {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(Some(account_type)) => Ok(account_type),
            Ok(None) => Err(AppError {
                message: Some("Invalid currency code".to_string()),
                cause: None,
                error_type: AppErrorType::PayloadValidationError,
            }),
        }
    }
}
