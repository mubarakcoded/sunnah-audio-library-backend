use crate::{core::AppError, models::banks::BanksList};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct Bank {
    pub bank_id: Uuid,
    pub bank_name: String,
    pub bank_code: String,
    pub slug: String,
    pub is_active: bool,
    pub bank_type: String,
    pub logo: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Bank {
    pub async fn get_banks_list(pool: &PgPool) -> Result<Vec<BanksList>, AppError> {
        let banks = sqlx::query_as!(
            BanksList,
            "SELECT bank_name, bank_code, slug, logo FROM banks WHERE is_active = true"
        )
        .fetch_all(pool)
        .await;

        match banks {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
        }
    }

    pub async fn get_local_bank(pool: &PgPool) -> Result<BanksList, AppError> {
        let bank_detail = sqlx::query_as!(
            BanksList,
            "SELECT bank_name, bank_code, slug, logo FROM banks WHERE is_active = true AND is_own_bank = true LIMIT 1"
        )
        .fetch_one(pool)
        .await;

        match bank_detail {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("Failed to execute query for local bank: {:?}", e);
                Err(AppError::db_error(e))
            }
        }
    }
}
