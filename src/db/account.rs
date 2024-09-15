use crate::{
    core::{AppError, AppErrorType},
    models::{
        account::{AccountBalanceDetails, AccountDetails, AccountInfo, AccountStatus, AccountTier},
        name_enquiry::NameEnquiryAccountData,
    },
};
use actix_web::web;
use bigdecimal::BigDecimal;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct CustomerAccountTbl {
    pub account_id: Uuid,
    pub customer_id: Uuid,
    pub account_name: String,
    pub account_number: String,
    pub phone_number: String,
    pub account_type_id: Uuid,
    pub account_tier_id: Uuid,
    pub currency_id: Uuid,
    pub account_code: String,
    pub is_active: bool,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl CustomerAccountTbl {
    pub async fn lock_account<'a>(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account_id: &Uuid,
    ) -> Result<(), AppError> {
        let lock_query = "SELECT 1 FROM customer_accounts WHERE account_id = $1 FOR UPDATE";
        sqlx::query(lock_query)
            .bind(account_id)
            .execute(transaction.as_mut())
            .await
            .map(|_| ())
            .map_err(|e| AppError::db_error(e))
    }

    pub async fn insert_account_details(
        db_transaction: &mut Transaction<'_, Postgres>,
        customer_id: &Uuid,
        account_name: &str,
        account_number: &str,
        phone_number: &str,
        account_type_id: &Uuid,
        account_tier_id: &Uuid,
        currency_id: &Uuid,
        account_code: &str,
    ) -> Result<CustomerAccountTbl, AppError> {
        let insert: Result<CustomerAccountTbl, sqlx::Error> = sqlx::query_as(
            r#"
       INSERT INTO customer_accounts (
        customer_id,
        account_name,
        account_number,
        phone_number,
        account_type_id,
        account_tier_id,
        currency_id,
        account_code
       ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) returning *
      "#,
        )
        .bind(customer_id)
        .bind(account_name)
        .bind(account_number)
        .bind(phone_number)
        .bind(account_type_id)
        .bind(account_tier_id)
        .bind(currency_id)
        .bind(account_code)
        .fetch_one(db_transaction.as_mut())
        .await;

        match insert {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(x) => Ok(x),
        }
    }

    pub async fn get_accounts_for_customer(
        db_pool: &PgPool,
        customer_id: &Uuid,
    ) -> Result<Vec<AccountDetails>, AppError> {
        let accounts = sqlx::query_as!(
            AccountDetails,
            r#"
            SELECT
                ca.account_id,
                ca.account_name,
                ca.account_number,
                c.currency_code,
                account_tiers.tier_name,
                account_tiers.max_account_balance,
                wb.balance,
                wb.available_balance
            FROM customer_accounts ca
            JOIN currencies c ON ca.currency_id = c.currency_id
            INNER JOIN account_tiers ON ca.account_tier_id = account_tiers.tier_id
            LEFT JOIN wallet_balance wb ON ca.account_id = wb.account_id
            WHERE ca.customer_id = $1
            AND wb.created_at = (
                SELECT MAX(created_at)
                FROM wallet_balance wb_sub
                WHERE wb_sub.account_id = ca.account_id
            )
            "#,
            customer_id
        )
        .fetch_all(db_pool)
        .await?;

        if accounts.is_empty() {
            return Err(AppError {
                message: Some("No accounts found for the given customer ID".to_string()),
                cause: None,
                error_type: AppErrorType::NotFoundError,
            });
        }

        Ok(accounts)
    }

    pub async fn find_account_by_customer_id_and_currency(
        db_pool: &PgPool,
        customer_id: &Uuid,
        currency_code: &String,
    ) -> Result<Option<CustomerAccountTbl>, AppError> {
        let account = sqlx::query_as!(
            CustomerAccountTbl,
            "SELECT * FROM customer_accounts WHERE customer_id = $1 AND currency_id = (SELECT currency_id FROM currencies WHERE currency_code = $2)",
            customer_id,
            currency_code
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(account)
    }

    pub async fn get_account_balance(
        db_pool: &PgPool,
        account_id: &Uuid,
    ) -> Result<AccountBalanceDetails, AppError> {
        let balance_details = sqlx::query_as!(
            AccountBalanceDetails,
            r#"
            SELECT
                balance,
                available_balance,
                c.currency_code
            FROM customer_accounts ca
            JOIN currencies c ON ca.currency_id = c.currency_id
            LEFT JOIN wallet_balance wb ON ca.account_id = wb.account_id
            WHERE ca.account_id = $1
            ORDER BY wb.created_at DESC
            LIMIT 1
            "#,
            account_id
        )
        .fetch_one(db_pool)
        .await?;
        Ok(balance_details)
    }

    pub async fn get_wallet_balance(
        db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account_id: Uuid,
    ) -> Result<BigDecimal, AppError> {
        let balance_query = r#"
            SELECT available_balance
            FROM wallet_balance
            WHERE account_id = $1
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        let balance = sqlx::query_scalar(&balance_query)
            .bind(account_id)
            .fetch_optional(db_transaction.as_mut())
            .await?;

        Ok(balance.unwrap_or_default())
    }

    pub async fn fetch_account_info(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account_id: &Uuid,
    ) -> Result<Option<AccountInfo>, AppError> {
        let account_info = sqlx::query_as!(
            AccountInfo,
            r#"
            SELECT
                customer_accounts.account_id, 
                customer_accounts.account_number, 
                customer_accounts.account_name, 
                wallet_balance.available_balance,
                account_tiers.tier_name,
                account_tiers.max_account_balance
            FROM 
                customer_accounts
            INNER JOIN 
                wallet_balance ON customer_accounts.account_id = wallet_balance.account_id
            INNER JOIN
                account_tiers ON customer_accounts.account_tier_id = account_tiers.tier_id
            WHERE 
                customer_accounts.account_id = $1
            ORDER BY 
                wallet_balance.created_at DESC
            LIMIT 1
            FOR UPDATE
            "#,
            account_id
        )
        .fetch_optional(transaction.as_mut())
        .await;

        match account_info {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(Some(data)) => Ok(Some(data)),
            Ok(None) => Ok(None),
        }
    }

    // async fn get_recent_available_balance(
    //     account_id: Uuid,
    //     db_pool: &PgPool,
    // ) -> Result<BigDecimal, AppError> {
    //     let result = sqlx::query("SELECT available_balance FROM wallet_balance WHERE account_id = ? ORDER BY created_at DESC LIMIT 1")
    //         .bind(account_id)
    //         .fetch_one(db_pool)
    //         .await?;
    //     Ok(result)
    // }

    pub async fn get_account_name_by_account_or_phone(
        db_pool: &web::Data<PgPool>,
        account_number: &str,
    ) -> Result<Option<NameEnquiryAccountData>, AppError> {
        let account_data = sqlx::query_as!(
            NameEnquiryAccountData,
            r#"
            SELECT account_name, account_id
            FROM customer_accounts
            WHERE account_number = $1 OR phone_number = $1
        "#,
            account_number
        )
        .fetch_optional(db_pool.get_ref())
        .await;

        match account_data {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(Some(data)) => Ok(Some(data)),
            Ok(None) => Err(AppError {
                message: Some("Account not found".to_string()),
                cause: None,
                error_type: AppErrorType::NotFoundError,
            }),
        }
    }

    pub async fn generate_unique_account_number(
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<String, sqlx::Error> {
        loop {
            let account_number = Self::generate_random_account_number("0012", 6);

            let exists = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM customer_accounts WHERE account_number = $1)",
                account_number
            )
            .fetch_one(transaction.as_mut())
            .await?;

            if let Some(false) = exists {
                return Ok(account_number);
            }
        }
    }

    fn generate_random_account_number(prefix: &str, num_digits: usize) -> String {
        let mut rng = rand::thread_rng();
        let mut account_number = prefix.to_owned();

        for _ in 0..num_digits {
            let digit: u8 = rng.gen_range(0..10);
            account_number.push(char::from_digit(digit as u32, 10).unwrap());
        }

        account_number
    }

    pub async fn deactivate_account(pool: &PgPool, account_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE customer_accounts SET is_active = false, status = $1 WHERE account_id = $2",
        )
        .bind(AccountStatus::Inactive.to_string())
        .bind(account_id)
        .execute(pool)
        .await;

        match result {
            Ok(rows_affected) => {
                if rows_affected.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(AppError {
                        message: Some("Failed to deactivate the account.".to_string()),
                        cause: None,
                        error_type: AppErrorType::PayloadValidationError,
                    })
                }
            }
            Err(e) => Err(AppError {
                message: Some(format!("Database error: {}", e)),
                cause: None,
                error_type: AppErrorType::DbError,
            }),
        }
    }

    pub async fn upgrade_account_tier(
        pool: &PgPool,
        account_id: Uuid,
        new_tier: Uuid,
    ) -> Result<CustomerAccountTbl, AppError> {
        let current_tier = sqlx::query_as::<_, AccountTier>(
            r#"
            SELECT * FROM account_tiers
            WHERE tier_id = (
                SELECT account_tier_id
                FROM customer_accounts
                WHERE account_id = $1
            )
            "#,
        )
        .bind(account_id)
        .fetch_one(pool)
        .await?;

        if current_tier.tier_id == new_tier {
            return Err(AppError {
                message: Some("Upgrade to a higher tier is required.".to_string()),
                cause: None,
                error_type: AppErrorType::PayloadValidationError,
            });
        }

        let result = sqlx::query_as::<_, CustomerAccountTbl>(
            r#"
            UPDATE customer_accounts
            SET account_tier_id = $1, updated_at = NOW()
            WHERE account_id = $2
            RETURNING *;
            "#,
        )
        .bind(new_tier)
        .bind(account_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn fetch_account_by_id(
        pool: &PgPool,
        account_id: Uuid,
    ) -> Option<CustomerAccountTbl> {
        let account_info = sqlx::query_as!(
            CustomerAccountTbl,
            r#"SELECT * FROM customer_accounts WHERE account_id = $1 LIMIT 1"#,
            account_id
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        account_info
    }
}
