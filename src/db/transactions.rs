use bigdecimal::BigDecimal;
use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
// use sqlx::{PgPool, Postgres, Transaction};
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row, Transaction};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::{
    core::AppError,
    models::transactions::{
        AdminDetailedTransactionResponse, AdminTransactionHistoryResponse, AdminTransactionResponse, TransactionData, TransactionDetail, TransactionHistoryResponse, TransactionMetrics, TransactionsResponse, TransferData, TxnHistoryBillPaymentData, TxnHistoryTransferData
    },
};

#[derive(Deserialize, Serialize, Debug)]
pub struct TransactionsTbl {
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub transaction_type: String,     // Debit or Credit
    pub transaction_category: String, //Transfer, Deposit, Reversal, Airtime Purchase, Data Purchase, Fee Debit, POS Transaction, ATM Withdrawal, Utility Payment etc.
    pub amount: Option<BigDecimal>,
    pub charges: Option<BigDecimal>,
    pub total_amount: Option<BigDecimal>,
    pub description: Option<String>,
    pub narration: Option<String>,
    pub channel: String,
    pub value_date: Option<chrono::NaiveDateTime>,
    pub transaction_date: chrono::NaiveDateTime,
    pub transaction_reference: String,
    pub payment_reference: Option<String>,
    pub status: String,
}

impl TransactionsTbl {
    pub async fn check_duplicate_transaction(
        db_transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        transactionn_reference: String,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query!(
            "
            SELECT EXISTS (
                SELECT 1
                FROM transactions
                WHERE transaction_reference = $1
            )
            ",
            transactionn_reference
        )
        .fetch_one(db_transaction.as_mut())
        .await?
        .exists
        .unwrap_or(false);

        Ok(exists)
    }

    pub async fn insert_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        transaction_data: &TransactionData,
    ) -> Result<Uuid, AppError> {
        let transaction_id = Uuid::new_v4();
        let account_id = transaction_data.account_id;
        let transaction_type = &transaction_data.transaction_type;
        let amount = &transaction_data.amount;
        let total_amount = &transaction_data.total_amount;
        let description = &transaction_data.description;
        let narration = &transaction_data.narration;
        let channel = &transaction_data.channel;
        let currency_code = &transaction_data.currency_code;
        let transaction_ref = &transaction_data.transaction_ref;
        let transaction_category = &transaction_data.transaction_category;
        let transaction_date = &transaction_data.transaction_date;
        let value_date = &transaction_data.value_date;
        let status = &transaction_data.status;

        let query = "
            INSERT INTO transactions (
                transaction_id, account_id, transaction_type, amount, total_amount, description, narration, channel, currency_code, transaction_reference, transaction_category, transaction_date, value_date, status
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            ) RETURNING transaction_id
        ";

        let result = sqlx::query(query)
            .bind(transaction_id)
            .bind(account_id)
            .bind(transaction_type)
            .bind(amount)
            .bind(total_amount)
            .bind(description)
            .bind(narration)
            .bind(channel)
            .bind(currency_code)
            .bind(transaction_ref)
            .bind(transaction_category.to_string())
            .bind(transaction_date)
            .bind(value_date)
            .bind(status)
            .execute(transaction.as_mut())
            .await?;

        Ok(transaction_id)
    }

    pub async fn insert_transfer_data(
        db_transaction: &mut Transaction<'_, Postgres>,
        transfer_data: &TransferData,
    ) -> Result<(), AppError> {
        let query = "
    INSERT INTO transfers (
        transfer_id,
        transaction_reference,
        source_account_number,
        source_account_name,
        source_bank_code,
        source_bank_name,
        beneficiary_account_number,
        beneficiary_account_name,
        beneficiary_bank_code,
        beneficiary_bank_name,
        transfer_type
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
    )";

        let result = sqlx::query(query)
            .bind(&transfer_data.transfer_id)
            .bind(&transfer_data.transaction_reference)
            .bind(&transfer_data.source_account_number)
            .bind(&transfer_data.source_account_name)
            .bind(&transfer_data.source_bank_code)
            .bind(&transfer_data.source_bank_name)
            .bind(&transfer_data.beneficiary_account_number)
            .bind(&transfer_data.beneficiary_account_name)
            .bind(&transfer_data.beneficiary_bank_code)
            .bind(&transfer_data.beneficiary_bank_name)
            .bind(&transfer_data.transfer_type)
            .execute(db_transaction.as_mut())
            .await?;

        Ok(())
    }

    pub async fn insert_new_balance_record(
        db_transaction: &mut Transaction<'_, Postgres>,
        account_id: Uuid,
        transaction_id: Uuid,
        amount: &BigDecimal,
        balance: &BigDecimal,
        available_balance: &BigDecimal,
        txn_date: DateTime<Local>,
    ) -> Result<(), AppError> {
        let query = "
            INSERT INTO wallet_balance (
                wallet_id,
                account_id,
                transaction_id,
                amount,
                balance,
                available_balance,
                created_at
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )
        ";

        let wallet_id = Uuid::new_v4();

        sqlx::query(query)
            .bind(wallet_id)
            .bind(account_id)
            .bind(transaction_id)
            .bind(amount)
            .bind(balance)
            .bind(available_balance)
            .bind(txn_date)
            .execute(db_transaction.as_mut())
            .await?;

        Ok(())
    }

    pub async fn insert_initial_balance(
        db_transaction: &mut Transaction<'_, Postgres>,
        account_id: &Uuid,
        transaction_id: Uuid,
        amount: BigDecimal,
        balance: BigDecimal,
        available_balance: BigDecimal,
    ) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO wallet_balance (
                account_id,
                transaction_id,
                amount,
                balance,
                available_balance
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
        "#;

        sqlx::query(query)
            .bind(account_id)
            .bind(transaction_id)
            .bind(amount)
            .bind(balance)
            .bind(available_balance)
            .execute(db_transaction.as_mut())
            .await?;

        Ok(())
    }

    pub async fn insert_ledger_record(
        transaction: &mut Transaction<'_, Postgres>,
        transaction_id: Uuid,
        account_id: Uuid,
        debit: &BigDecimal,
        credit: &BigDecimal,
        balance: &BigDecimal,
        description: &String,
        txn_date: DateTime<Local>,
    ) -> Result<(), AppError> {
        let query = "
            INSERT INTO ledger (
                transaction_id,
                account_id,
                debit,
                credit,
                balance,
                description,
                created_at
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )
        ";

        sqlx::query(query)
            .bind(transaction_id)
            .bind(account_id)
            .bind(debit)
            .bind(credit)
            .bind(balance)
            .bind(description)
            .bind(txn_date)
            .execute(transaction.as_mut())
            .await?;

        Ok(())
    }

    // pub async fn get_recent_transactions(
    //     pool: &PgPool,
    //     account_id: Uuid,
    //     category: Option<String>,
    // ) -> Result<Vec<Transactions>, AppError> {
    //     let recent_transactions = sqlx::query_as(
    //         r#"
    //             SELECT
    //                 transaction_id,
    //                 transaction_type,
    //                 total_amount,
    //                 description,
    //                 currency_code,
    //                 transaction_reference,
    //                 transaction_category,
    //                 transaction_date,
    //                 status
    //             FROM
    //                 transactions
    //             WHERE
    //                 account_id = $1
    //             ORDER BY
    //                 transaction_date DESC
    //             LIMIT 10
    //         "#,
    //     )
    //     .bind(account_id)
    //     .fetch_all(pool)
    //     .await?;

    //     Ok(recent_transactions)
    // }

    pub async fn get_recent_transactions(
        db_pool: &PgPool,
        account_id: Uuid,
        category: Option<String>,
    ) -> Result<Vec<TransactionsResponse>, AppError> {
        let mut data_query = String::from("SELECT
            t.transaction_id,
            t.amount,
            t.total_amount,
            t.status,
            t.transaction_type,
            t.transaction_category,
            t.description,
            t.narration,
            t.session_id,
            t.transaction_date,
            t.transaction_reference,
            CASE
                WHEN t.transaction_category IN ('Airtime Purchase', 'Data Purchase') THEN mn.logo_url
                WHEN t.transaction_category = 'Electricity Purchase' THEN ed.logo_url
                WHEN t.transaction_category = 'Cable TV' THEN cp.logo_url
                WHEN t.transaction_category = 'Transfer' THEN
                    CASE
                        WHEN t.transaction_type = 'Credit' THEN sb.logo
                        WHEN t.transaction_type = 'Debit' THEN db.logo
                    END
            END AS logo_url,
            tf.transfer_id,
            tf.transfer_type,
            tf.source_account_number,
            tf.source_account_name,
            tf.source_bank_name,
            tf.beneficiary_account_number,
            tf.beneficiary_account_name,
            tf.beneficiary_bank_name,
            bp.bill_payment_id,
            bp.biller_name,
            bp.plan_name,
            bp.bills_category,
            bp.phone_number,
            bp.iuc_smartcard_number,
            bp.meter_number,
            bp.purchased_token
        FROM
            transactions t
        LEFT JOIN
            transfers tf ON t.transaction_reference = tf.transaction_reference
        LEFT JOIN
            bills_payments bp ON t.transaction_id = bp.transaction_id
        LEFT JOIN
            mobile_networks mn ON bp.biller_id = mn.network_id AND t.transaction_category IN ('Airtime Purchase', 'Data Purchase')
        LEFT JOIN
            electricity_discos ed ON bp.biller_id = ed.disco_id AND t.transaction_category = 'Electricity Purchase'
        LEFT JOIN
            cable_providers cp ON bp.biller_id = cp.provider_id AND t.transaction_category = 'Cable TV'
        LEFT JOIN
            banks sb ON tf.source_bank_code = sb.bank_code AND t.transaction_category = 'Transfer' AND t.transaction_type = 'Credit'
        LEFT JOIN
            banks db ON tf.beneficiary_bank_code = db.bank_code AND t.transaction_category = 'Transfer' AND t.transaction_type = 'Debit'
        WHERE
            t.account_id = $1");

        if let Some(category) = category {
            data_query.push_str(&format!(" AND t.transaction_category = '{}' ", category));
        }

        data_query.push_str(" ORDER BY t.transaction_date DESC LIMIT 10");

        let rows: Vec<PgRow> = sqlx::query(&data_query.as_str())
            .bind(account_id)
            .fetch_all(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let transaction_history_results: Vec<TransactionsResponse> = rows
            .into_iter()
            .map(|row| {
                let transfer_data = if row.get::<Option<Uuid>, _>("transfer_id").is_some() {
                    Some(TxnHistoryTransferData {
                        transfer_type: row.get("transfer_type"),
                        source_account_number: row.get("source_account_number"),
                        source_account_name: row.get("source_account_name"),
                        source_bank_name: row.get("source_bank_name"),
                        beneficiary_account_number: row.get("beneficiary_account_number"),
                        beneficiary_account_name: row.get("beneficiary_account_name"),
                        beneficiary_bank_name: row.get("beneficiary_bank_name"),
                    })
                } else {
                    None
                };

                let bill_payment_data = if row.get::<Option<Uuid>, _>("bill_payment_id").is_some() {
                    Some(TxnHistoryBillPaymentData {
                        biller_name: row.get("biller_name"),
                        plan_name: row.get("plan_name"),
                        bills_category: row.get("bills_category"),
                        phone_number: row.get("phone_number"),
                        iuc_smartcard_number: row.get("iuc_smartcard_number"),
                        meter_number: row.get("meter_number"),
                        purchased_token: row.get("purchased_token"),
                    })
                } else {
                    None
                };

                TransactionsResponse {
                    transaction_id: row.get("transaction_id"),
                    transaction_type: row.get("transaction_type"),
                    transaction_category: row.get("transaction_category"),
                    amount: row.get("amount"),
                    total_amount: row.get("total_amount"),
                    description: row.get("description"),
                    narration: row.get("narration"),
                    session_id: row.get("session_id"),
                    transaction_date: row.get("transaction_date"),
                    transaction_reference: row.get("transaction_reference"),
                    status: row.get("status"),
                    logo_url: row.get("logo_url"),
                    transfer_data,
                    bill_payment_data,
                }
            })
            .collect();

        Ok(transaction_history_results)
    }

    pub async fn get_transactions_history(
        db_pool: &PgPool,
        account_id: Uuid,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        transaction_type: Option<String>,
        category: Option<String>,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<TransactionHistoryResponse, AppError> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);

        let mut data_query = String::from("SELECT
            t.transaction_id,
            t.amount,
            t.total_amount,
            t.status,
            t.transaction_type,
            t.transaction_category,
            t.description,
            t.narration,
            t.session_id,
            t.transaction_date,
            t.transaction_reference,
            CASE
                WHEN t.transaction_category IN ('Airtime Purchase', 'Data Purchase') THEN mn.logo_url
                WHEN t.transaction_category = 'Electricity Purchase' THEN ed.logo_url
                WHEN t.transaction_category = 'Cable TV' THEN cp.logo_url
                WHEN t.transaction_category = 'Transfer' THEN
                    CASE
                        WHEN t.transaction_type = 'Credit' THEN sb.logo
                        WHEN t.transaction_type = 'Debit' THEN db.logo
                    END
            END AS logo_url,
            tf.transfer_id,
            tf.transfer_type,
            tf.source_account_number,
            tf.source_account_name,
            tf.source_bank_name,
            tf.beneficiary_account_number,
            tf.beneficiary_account_name,
            tf.beneficiary_bank_name,
            bp.bill_payment_id,
            bp.biller_name,
            bp.plan_name,
            bp.bills_category,
            bp.phone_number,
            bp.iuc_smartcard_number,
            bp.meter_number,
            bp.purchased_token
        FROM
            transactions t
        LEFT JOIN
            transfers tf ON t.transaction_reference = tf.transaction_reference
        LEFT JOIN
            bills_payments bp ON t.transaction_id = bp.transaction_id
        LEFT JOIN
            mobile_networks mn ON bp.biller_id = mn.network_id AND t.transaction_category IN ('Airtime Purchase', 'Data Purchase')
        LEFT JOIN
            electricity_discos ed ON bp.biller_id = ed.disco_id AND t.transaction_category = 'Electricity Purchase'
        LEFT JOIN
            cable_providers cp ON bp.biller_id = cp.provider_id AND t.transaction_category = 'Cable TV'
        LEFT JOIN
            banks sb ON tf.source_bank_code = sb.bank_code AND t.transaction_category = 'Transfer' AND t.transaction_type = 'Credit'
        LEFT JOIN
            banks db ON tf.beneficiary_bank_code = db.bank_code AND t.transaction_category = 'Transfer' AND t.transaction_type = 'Debit'
        WHERE
            t.account_id = $1");

        let mut total_count_query =
            String::from("SELECT Count(*) AS total_count FROM transactions WHERE account_id = $1");

        if let Some(transaction_type) = transaction_type {
            data_query.push_str(&format!(
                " AND t.transaction_type = '{}' ",
                transaction_type
            ));
            total_count_query.push_str(&format!(" AND transaction_type = '{}' ", transaction_type));
        }

        if let Some(category) = category {
            data_query.push_str(&format!(" AND t.transaction_category = '{}' ", category));
            total_count_query.push_str(&format!(" AND transaction_category = '{}' ", category));
        }

        if let (Some(start_date), Some(end_date)) = (start_date, end_date) {
            data_query.push_str(&format!(
                " AND t.transaction_date BETWEEN '{}' AND '{}' ",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            ));

            total_count_query.push_str(&format!(
                " AND transaction_date BETWEEN '{}' AND '{}' ",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            ));
        }

        data_query.push_str(" ORDER BY t.transaction_date DESC ");

        let offset = (page - 1) * page_size;
        data_query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        // let paginated_data = sqlx::query_as::<_, Transactions>(data_query.as_str())
        let rows: Vec<PgRow> = sqlx::query(&data_query.as_str())
            .bind(account_id)
            .fetch_all(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let transaction_history_results: Vec<TransactionsResponse> = rows
            .into_iter()
            .map(|row| {
                let transfer_data = if row.get::<Option<Uuid>, _>("transfer_id").is_some() {
                    Some(TxnHistoryTransferData {
                        transfer_type: row.get("transfer_type"),
                        source_account_number: row.get("source_account_number"),
                        source_account_name: row.get("source_account_name"),
                        source_bank_name: row.get("source_bank_name"),
                        beneficiary_account_number: row.get("beneficiary_account_number"),
                        beneficiary_account_name: row.get("beneficiary_account_name"),
                        beneficiary_bank_name: row.get("beneficiary_bank_name"),
                    })
                } else {
                    None
                };

                let bill_payment_data = if row.get::<Option<Uuid>, _>("bill_payment_id").is_some() {
                    Some(TxnHistoryBillPaymentData {
                        biller_name: row.get("biller_name"),
                        plan_name: row.get("plan_name"),
                        bills_category: row.get("bills_category"),
                        phone_number: row.get("phone_number"),
                        iuc_smartcard_number: row.get("iuc_smartcard_number"),
                        meter_number: row.get("meter_number"),
                        purchased_token: row.get("purchased_token"),
                    })
                } else {
                    None
                };

                TransactionsResponse {
                    transaction_id: row.get("transaction_id"),
                    transaction_type: row.get("transaction_type"),
                    transaction_category: row.get("transaction_category"),
                    amount: row.get("amount"),
                    total_amount: row.get("total_amount"),
                    description: row.get("description"),
                    narration: row.get("narration"),
                    session_id: row.get("session_id"),
                    transaction_date: row.get("transaction_date"),
                    transaction_reference: row.get("transaction_reference"),
                    status: row.get("status"),
                    logo_url: row.get("logo_url"),
                    transfer_data,
                    bill_payment_data,
                }
            })
            .collect();

        let total_count: i64 = sqlx::query_scalar(&total_count_query)
            .bind(account_id)
            .fetch_one(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let response = TransactionHistoryResponse {
            transactions: transaction_history_results,
            total_count: total_count as u64,
            page: page,
            page_size: page_size,
        };

        Ok(response)
    }

    pub async fn fetch_account_statement_working(
        db_pool: &PgPool,
        account_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<TransactionDetail>, AppError> {
        // let end_datetime = end_date.and_hms(23, 59, 59); // Adjust end date to end of the day

        let query = format!(
            r#"
            SELECT 
                t.transaction_id,
                t.transaction_type,
                t.total_amount as amount,
                t.narration,
                t.transaction_date,
                wb.available_balance + t.total_amount AS balance_before,
                wb.available_balance AS balance_after
            FROM transactions t
            JOIN wallet_balance wb 
                ON t.transaction_id = wb.transaction_id
            WHERE t.account_id = '{}'
            AND t.transaction_date::timestamptz::date BETWEEN '{}' AND '{}'
            ORDER BY t.transaction_date ASC
            "#,
            account_id,
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d")
        );

        let transactions = sqlx::query_as::<_, TransactionDetail>(&query)
            .fetch_all(db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::db_error(e.to_string())
            })?;

        Ok(transactions)
    }

    pub async fn fetch_account_statement(
        db_pool: &PgPool,
        account_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<TransactionDetail>, AppError> {
        let transactions = sqlx::query_as(
            r#"
            SELECT 
                t.transaction_id,
                t.transaction_type,
                t.total_amount as amount,
                t.narration,
                t.transaction_date,
                COALESCE(
                    CASE 
                        WHEN t.transaction_type = 'Debit' THEN wb.available_balance + t.total_amount
                        WHEN t.transaction_type = 'Credit' THEN wb.available_balance - t.total_amount
                        ELSE wb.available_balance
                    END, 
                    0
                ) AS balance_before,
                COALESCE(wb.available_balance, 0) AS balance_after
            FROM transactions t
            JOIN wallet_balance wb 
                ON t.transaction_id = wb.transaction_id
            WHERE t.account_id = $1
              AND t.transaction_date::DATE BETWEEN $2::DATE AND $3::DATE
            ORDER BY t.transaction_date ASC
            "#,
        )
        .bind(account_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(db_pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_all_transactions(
        db_pool: &PgPool,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        status: Option<String>,
        transaction_reference: Option<String>,
        transaction_type: Option<String>,
        category: Option<String>,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<AdminTransactionHistoryResponse, AppError> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(25);

        let mut data_query = String::from("SELECT * FROM transactions");

        let mut total_count_query =
            String::from("SELECT Count(*) AS total_count FROM transactions");

        if let Some(start_date) = start_date {
            data_query.push_str(&format!(
                " WHERE transaction_date >= '{}' ",
                start_date.format("%Y-%m-%d")
            ));
            total_count_query.push_str(&format!(
                " WHERE transaction_date >= '{}' ",
                start_date.format("%Y-%m-%d")
            ));
        }

        if let Some(end_date) = end_date {
            data_query.push_str(&format!(
                " AND transaction_date <= '{}' ",
                end_date.format("%Y-%m-%d")
            ));
            total_count_query.push_str(&format!(
                " AND transaction_date <= '{}' ",
                end_date.format("%Y-%m-%d")
            ));
        }

        if let Some(transaction_type) = transaction_type {
            data_query.push_str(&format!(" AND transaction_type = '{}' ", transaction_type));
            total_count_query.push_str(&format!(" AND transaction_type = '{}' ", transaction_type));
        }

        if let Some(category) = category {
            data_query.push_str(&format!(" AND transaction_category = '{}' ", category));
            total_count_query.push_str(&format!(" AND transaction_category = '{}' ", category));
        }

        if let Some(status) = status {
            data_query.push_str(&format!(" AND status = '{}' ", status));
            total_count_query.push_str(&format!(" AND status = '{}' ", status));
        }

        if let Some(transaction_reference) = transaction_reference {
            data_query.push_str(&format!(
                " AND transaction_reference = '{}' ",
                transaction_reference
            ));
            total_count_query.push_str(&format!(
                " AND transaction_reference = '{}' ",
                transaction_reference
            ));
        }

        data_query.push_str(" ORDER BY transaction_date DESC ");

        let offset = (page - 1) * page_size;
        data_query.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        let paginated_data = sqlx::query_as::<_, AdminTransactionResponse>(data_query.as_str())
            .fetch_all(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let total_count: i64 = sqlx::query_scalar(&total_count_query)
            .fetch_one(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let response = AdminTransactionHistoryResponse {
            transactions: paginated_data,
            total_count: total_count as u64,
            page: page,
            page_size: page_size,
        };

        Ok(response)
    }

    pub async fn get_transaction_by_id(
        db_pool: &PgPool,
        transaction_id: Uuid,
    ) -> Result<AdminDetailedTransactionResponse, AppError> {
        let query = r#"
        SELECT
            t.transaction_id,
            t.account_id,
            t.amount,
            t.total_amount,
            t.status,
            t.transaction_type,
            t.transaction_category,
            t.description,
            t.narration,
            t.session_id,
            t.transaction_date,
            t.transaction_reference,
            tf.transfer_id,
            tf.transfer_type,
            tf.source_account_number,
            tf.source_account_name,
            tf.source_bank_name,
            tf.beneficiary_account_number,
            tf.beneficiary_account_name,
            tf.beneficiary_bank_name,
            bp.bill_payment_id,
            bp.biller_name,
            bp.plan_name,
            bp.bills_category,
            bp.phone_number,
            bp.iuc_smartcard_number,
            bp.meter_number,
            bp.purchased_token
        FROM
            transactions t
        LEFT JOIN
            transfers tf ON t.transaction_reference = tf.transaction_reference
        LEFT JOIN
            bills_payments bp ON t.transaction_id = bp.transaction_id
        WHERE
            t.transaction_id = $1
    "#;

        let row: PgRow = sqlx::query(query)
            .bind(transaction_id)
            .fetch_one(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let transfer_data = if row.get::<Option<Uuid>, _>("transfer_id").is_some() {
            Some(TxnHistoryTransferData {
                transfer_type: row.get("transfer_type"),
                source_account_number: row.get("source_account_number"),
                source_account_name: row.get("source_account_name"),
                source_bank_name: row.get("source_bank_name"),
                beneficiary_account_number: row.get("beneficiary_account_number"),
                beneficiary_account_name: row.get("beneficiary_account_name"),
                beneficiary_bank_name: row.get("beneficiary_bank_name"),
            })
        } else {
            None
        };

        let bill_payment_data = if row.get::<Option<Uuid>, _>("bill_payment_id").is_some() {
            Some(TxnHistoryBillPaymentData {
                biller_name: row.get("biller_name"),
                plan_name: row.get("plan_name"),
                bills_category: row.get("bills_category"),
                phone_number: row.get("phone_number"),
                iuc_smartcard_number: row.get("iuc_smartcard_number"),
                meter_number: row.get("meter_number"),
                purchased_token: row.get("purchased_token"),
            })
        } else {
            None
        };

        Ok(AdminDetailedTransactionResponse {
            transaction_id: row.get("transaction_id"),
            account_id: row.get("account_id"),
            transaction_type: row.get("transaction_type"),
            transaction_category: row.get("transaction_category"),
            amount: row.get("amount"),
            total_amount: row.get("total_amount"),
            description: row.get("description"),
            narration: row.get("narration"),
            session_id: row.get("session_id"),
            transaction_date: row.get("transaction_date"),
            transaction_reference: row.get("transaction_reference"),
            status: row.get("status"),
            transfer_data,
            bill_payment_data,
        })
    }

    pub async fn get_transaction_metrics(db_pool: &PgPool) -> Result<TransactionMetrics, AppError> {
        let total_transactions: i64 = sqlx::query_scalar("SELECT Count(*) FROM transactions")
            .fetch_one(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let total_pending_transactions: i64 =
            sqlx::query_scalar("SELECT Count(*) FROM transactions WHERE status = 'pending'")
                .fetch_one(db_pool)
                .await
                .map_err(AppError::db_error)?;

        let total_successful_transactions: i64 =
            sqlx::query_scalar("SELECT Count(*) FROM transactions WHERE status = 'success'")
                .fetch_one(db_pool)
                .await
                .map_err(AppError::db_error)?;

        let total_failed_transactions: i64 =
            sqlx::query_scalar("SELECT Count(*) FROM transactions WHERE status = 'failed'")
                .fetch_one(db_pool)
                .await
                .map_err(AppError::db_error)?;

        let total_transactions_volume: BigDecimal = sqlx::query_scalar("SELECT SUM(amount) FROM transactions")
            .fetch_one(db_pool)
            .await
            .map_err(AppError::db_error)?;

        let transaction_metrics = TransactionMetrics {
            total_transactions,
            total_pending_transactions,
            total_successful_transactions,
            total_failed_transactions,
            total_transactions_volume,
        };

        Ok(transaction_metrics)
    }
}
