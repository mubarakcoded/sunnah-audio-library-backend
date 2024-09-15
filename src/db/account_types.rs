use crate::core::{AppError, AppErrorType};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct AccountTypesTbl {
    pub account_type_id: Uuid,
    pub account_type: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl AccountTypesTbl {

    #[tracing::instrument(name = "Getting account types details by account_type name", skip(db_pool))]
    pub async fn find_by_account_type(db_pool: &PgPool, account_type: String) -> Result<AccountTypesTbl, AppError> {
        match sqlx::query_as::<_, AccountTypesTbl>("SELECT * FROM account_types WHERE account_type = $1")
            .bind(&account_type)
            .fetch_optional(db_pool)
            .await
        {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(Some(account_type)) => Ok(account_type),
            Ok(None) => Err(AppError {
                message: Some("Invalid account type".to_string()),
                cause: None,
                error_type: AppErrorType::PayloadValidationError,
            }),
        }
    }
}
