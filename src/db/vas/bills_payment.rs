use bigdecimal::BigDecimal;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::core::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct BillsPaymentsTbl {
    pub transaction_id: Uuid,
    pub biller_id: Uuid,
    pub biller_name: String,
    pub plan_name: String,
    pub bills_category: String,
    pub phone_number: Option<String>,
    pub iuc_smartcard_number: Option<String>,
    pub meter_number: Option<String>,
    pub email_address: Option<String>,
    pub biller_reference_number: Option<String>,
    pub amount: BigDecimal,
    pub discount: Option<BigDecimal>,
    pub charges: Option<BigDecimal>,
    pub payment_date: DateTime<Local>,
    pub payment_reference: Option<String>,
    pub purchased_token: Option<String>,
    pub status: String,
    pub details: serde_json::Value,
}

impl BillsPaymentsTbl {
    pub async fn insert_bill_payment(
        db_transaction: &mut Transaction<'_, Postgres>,
        bill_payment: &BillsPaymentsTbl,
    ) -> Result<(), AppError> {
        
        sqlx::query!(
            r#"
            INSERT INTO bills_payments (
                transaction_id,
                biller_id,
                biller_name,
                plan_name,
                bills_category,
                phone_number,
                iuc_smartcard_number,
                meter_number,
                email_address,
                biller_reference_number,
                amount,
                discount,
                charges,
                payment_date,
                payment_reference,
                purchased_token,
                status,
                details
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            "#,
            bill_payment.transaction_id,
            bill_payment.biller_id,
            bill_payment.biller_name,
            bill_payment.plan_name,
            bill_payment.bills_category,
            bill_payment.phone_number,
            bill_payment.iuc_smartcard_number,
            bill_payment.meter_number,
            bill_payment.email_address,
            bill_payment.biller_reference_number,
            bill_payment.amount,
            bill_payment.discount,
            bill_payment.charges,
            bill_payment.payment_date,
            bill_payment.payment_reference,
            bill_payment.purchased_token,
            bill_payment.status,
            bill_payment.details
        )
        .execute(db_transaction.as_mut())
        .await?;

        Ok(())
    }
}

