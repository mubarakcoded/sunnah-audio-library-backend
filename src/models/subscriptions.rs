use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, NaiveDateTime};
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionPlan {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub duration_type: String, // monthly, quarterly, bi_annually, yearly
    pub duration_months: i32,
    pub price: BigDecimal,
    pub currency: String,
    pub features: Option<serde_json::Value>, // JSON array of features
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub id: i32,
    pub user_id: i32,
    pub subscription_plan_id: i32,
    pub status: String, // pending, active, expired, cancelled
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub payment_method: Option<String>,
    pub transaction_reference: Option<String>,
    pub payment_amount: BigDecimal,
    pub payment_currency: String,
    pub payment_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscriptionWithPlan {
    pub id: i32,
    pub user_id: i32,
    pub subscription_plan_id: i32,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub payment_method: Option<String>,
    pub transaction_reference: Option<String>,
    pub payment_amount: BigDecimal,
    pub payment_currency: String,
    pub payment_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub plan: SubscriptionPlan,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub subscription_plan_id: i32,
    pub payment_method: String,
    pub transaction_reference: String,
    pub payment_amount: BigDecimal,
    pub payment_currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifySubscriptionRequest {
    pub status: String, // active, cancelled
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionPlanSummary {
    pub id: i32,
    pub name: String,
    pub duration_type: String,
    pub duration_months: i32,
    pub price: BigDecimal,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscriptionWithPlanSummary {
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub payment_method: Option<String>,
    pub transaction_reference: Option<String>,
    pub payment_date: Option<NaiveDateTime>,
    pub plan: SubscriptionPlanSummary,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionStatus {
    pub has_active_subscription: bool,
    pub current_subscription: Option<UserSubscriptionWithPlanSummary>,
    pub subscription_expires_at: Option<NaiveDate>,
    pub days_remaining: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscriptionMinimal {
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub payment_method: Option<String>,
    pub transaction_reference: Option<String>,
    pub payment_amount: BigDecimal,
    pub payment_date: Option<NaiveDateTime>,
    pub plan_name: String,
    pub duration_type: String,
    pub duration_months: i32,
}
