use crate::core::{AppError, AppErrorType};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct AccountTiersTbl {
    pub tier_id: Uuid,
    pub tier_name: String,
    pub tier_slug: String,
    pub max_account_balance: Option<BigDecimal>,
    pub daily_transfer_limit: Option<BigDecimal>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl AccountTiersTbl {

    #[tracing::instrument(name = "Getting account tier details by slug", skip(db_pool))]
    pub async fn find_by_tier_slug(db_pool: &PgPool, tier_slug: String) -> Result<AccountTiersTbl, AppError> {
        match sqlx::query_as::<_, AccountTiersTbl>("SELECT * FROM account_tiers WHERE tier_slug = $1")
            .bind(&tier_slug)
            .fetch_optional(db_pool)
            .await
        {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(Some(account_tier)) => Ok(account_tier),
            Ok(None) => Err(AppError {
                message: Some("Invalid account tier name".to_string()),
                cause: None,
                error_type: AppErrorType::PayloadValidationError,
            }),
        }
    }
}
