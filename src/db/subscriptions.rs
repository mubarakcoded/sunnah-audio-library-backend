use crate::core::AppError;
use crate::models::subscriptions::{
    SubscriptionPlan, UserSubscription, CreateSubscriptionRequest,
    VerifySubscriptionRequest, SubscriptionStatus, UserSubscriptionWithPlanSummary,
    SubscriptionPlanSummary,
};
use sqlx::MySqlPool;
use chrono::Utc;

// Get all subscription plans
pub async fn get_all_subscription_plans(
    pool: &MySqlPool,
) -> Result<Vec<SubscriptionPlan>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, description, duration_type, duration_months, 
               price, currency, features, is_active, sort_order,
               created_at, updated_at
        FROM tbl_subscription_plans
        WHERE is_active = 1
        ORDER BY sort_order ASC, price ASC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let plans = rows
        .into_iter()
        .map(|row| SubscriptionPlan {
            id: row.id,
            name: row.name,
            description: row.description,
            duration_type: row.duration_type,
            duration_months: row.duration_months,
            price: row.price,
            currency: row.currency,
            features: row.features,
            is_active: row.is_active != 0,
            sort_order: row.sort_order.unwrap_or(0),
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
        })
        .collect();

    Ok(plans)
}

// Get subscription plan by ID
pub async fn get_subscription_plan_by_id(
    pool: &MySqlPool,
    plan_id: i32,
) -> Result<SubscriptionPlan, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, name, description, duration_type, duration_months, 
               price, currency, features, is_active, sort_order,
               created_at, updated_at
        FROM tbl_subscription_plans
        WHERE id = ? AND is_active = 1
        "#,
        plan_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(SubscriptionPlan {
        id: row.id,
        name: row.name,
        description: row.description,
        duration_type: row.duration_type,
        duration_months: row.duration_months,
        price: row.price,
        currency: row.currency,
        features: row.features,
        is_active: row.is_active != 0,
        sort_order: row.sort_order.unwrap_or(0),
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    })
}

// Create user subscription
pub async fn create_user_subscription(
    pool: &MySqlPool,
    user_id: i32,
    request: &CreateSubscriptionRequest,
) -> Result<UserSubscription, AppError> {
    let now = Utc::now().naive_utc();
    let currency = request.payment_currency.as_deref().unwrap_or("CFA");

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_user_subscriptions 
        (user_id, subscription_plan_id, status, payment_method, transaction_reference, 
         payment_amount, payment_currency, payment_date, created_at, updated_at)
        VALUES (?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        request.subscription_plan_id,
        request.payment_method,
        request.transaction_reference,
        request.payment_amount,
        currency,
        now,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let subscription_id = result.last_insert_id() as i32;
    get_user_subscription_by_id(pool, subscription_id).await
}

// Get user subscription by ID
pub async fn get_user_subscription_by_id(
    pool: &MySqlPool,
    subscription_id: i32,
) -> Result<UserSubscription, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_plan_id, status, start_date, end_date,
               payment_method, transaction_reference, payment_amount, payment_currency,
               payment_date, notes, created_at, updated_at
        FROM tbl_user_subscriptions
        WHERE id = ?
        "#,
        subscription_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(UserSubscription {
        id: row.id,
        user_id: row.user_id,
        subscription_plan_id: row.subscription_plan_id,
        status: row.status,
        start_date: row.start_date,
        end_date: row.end_date,
        payment_method: row.payment_method,
        transaction_reference: row.transaction_reference,
        payment_amount: row.payment_amount,
        payment_currency: row.payment_currency,
        payment_date: row.payment_date,
        notes: row.notes,
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    })
}

// Get user subscriptions
pub async fn get_user_subscriptions(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<UserSubscription>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_plan_id, status, start_date, end_date,
               payment_method, transaction_reference, payment_amount, payment_currency,
               payment_date, notes, created_at, updated_at
        FROM tbl_user_subscriptions
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let subscriptions = rows
        .into_iter()
        .map(|row| UserSubscription {
            id: row.id,
            user_id: row.user_id,
            subscription_plan_id: row.subscription_plan_id,
            status: row.status,
            start_date: row.start_date,
            end_date: row.end_date,
            payment_method: row.payment_method,
            transaction_reference: row.transaction_reference,
            payment_amount: row.payment_amount,
            payment_currency: row.payment_currency,
            payment_date: row.payment_date,
            notes: row.notes,
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
        })
        .collect();

    Ok(subscriptions)
}

// Get user active subscription with plan summary
pub async fn get_user_active_subscription_with_plan(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Option<UserSubscriptionWithPlanSummary>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT 
            us.status, us.start_date, us.end_date,
            us.payment_method, us.transaction_reference,
            us.payment_date,
            sp.id as plan_id, sp.name as plan_name, sp.duration_type, sp.duration_months,
            sp.price as plan_price, sp.currency as plan_currency
        FROM tbl_user_subscriptions us
        JOIN tbl_subscription_plans sp ON us.subscription_plan_id = sp.id
        WHERE us.user_id = ? AND us.status = 'active' 
        AND (us.end_date IS NULL OR us.end_date >= CURDATE())
        ORDER BY us.created_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    if let Some(row) = row {
        Ok(Some(UserSubscriptionWithPlanSummary {
            status: row.status,
            start_date: row.start_date,
            end_date: row.end_date,
            payment_method: row.payment_method,
            transaction_reference: row.transaction_reference,
            payment_date: row.payment_date,
            plan: SubscriptionPlanSummary {
                id: row.plan_id,
                name: row.plan_name,
                duration_type: row.duration_type,
                duration_months: row.duration_months,
                price: row.plan_price,
                currency: row.plan_currency,
            },
        }))
    } else {
        Ok(None)
    }
}

// Keep the original function for backward compatibility
pub async fn get_user_active_subscription(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Option<UserSubscription>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_plan_id, status, start_date, end_date,
               payment_method, transaction_reference, payment_amount, payment_currency,
               payment_date, notes, created_at, updated_at
        FROM tbl_user_subscriptions
        WHERE user_id = ? AND status = 'active' 
        AND (end_date IS NULL OR end_date >= CURDATE())
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::db_error)?;

    if let Some(row) = row {
        Ok(Some(UserSubscription {
            id: row.id,
            user_id: row.user_id,
            subscription_plan_id: row.subscription_plan_id,
            status: row.status,
            start_date: row.start_date,
            end_date: row.end_date,
            payment_method: row.payment_method,
            transaction_reference: row.transaction_reference,
            payment_amount: row.payment_amount,
            payment_currency: row.payment_currency,
            payment_date: row.payment_date,
            notes: row.notes,
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
        }))
    } else {
        Ok(None)
    }
}

// Verify user subscription (admin function) - Auto-calculates dates based on plan
pub async fn verify_user_subscription(
    pool: &MySqlPool,
    subscription_id: i32,
    request: &VerifySubscriptionRequest,
) -> Result<UserSubscription, AppError> {
    let now = Utc::now().naive_utc();

    if request.status == "active" {
        // Get subscription plan details to calculate dates
        let subscription_with_plan = sqlx::query!(
            r#"
            SELECT us.id, sp.duration_months
            FROM tbl_user_subscriptions us
            JOIN tbl_subscription_plans sp ON us.subscription_plan_id = sp.id
            WHERE us.id = ?
            "#,
            subscription_id
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::db_error)?;

        // Calculate start and end dates based on plan duration
        let start_date = chrono::Utc::now().date_naive();
        let end_date = start_date + chrono::Duration::days(subscription_with_plan.duration_months as i64 * 30);

        sqlx::query!(
            r#"
            UPDATE tbl_user_subscriptions 
            SET status = ?, start_date = ?, end_date = ?, notes = ?, updated_at = ?
            WHERE id = ?
            "#,
            request.status,
            start_date,
            end_date,
            request.notes,
            now,
            subscription_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    } else {
        // For cancelled status, don't update dates
        sqlx::query!(
            r#"
            UPDATE tbl_user_subscriptions 
            SET status = ?, notes = ?, updated_at = ?
            WHERE id = ?
            "#,
            request.status,
            request.notes,
            now,
            subscription_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    get_user_subscription_by_id(pool, subscription_id).await
}

// Get pending subscriptions (admin function)
pub async fn get_pending_subscriptions(
    pool: &MySqlPool,
) -> Result<Vec<UserSubscription>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_plan_id, status, start_date, end_date,
               payment_method, transaction_reference, payment_amount, payment_currency,
               payment_date, notes, created_at, updated_at
        FROM tbl_user_subscriptions
        WHERE status = 'pending'
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let subscriptions = rows
        .into_iter()
        .map(|row| UserSubscription {
            id: row.id,
            user_id: row.user_id,
            subscription_plan_id: row.subscription_plan_id,
            status: row.status,
            start_date: row.start_date,
            end_date: row.end_date,
            payment_method: row.payment_method,
            transaction_reference: row.transaction_reference,
            payment_amount: row.payment_amount,
            payment_currency: row.payment_currency,
            payment_date: row.payment_date,
            notes: row.notes,
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
        })
        .collect();

    Ok(subscriptions)
}

// Get user subscription status with plan details
pub async fn get_user_subscription_status(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<SubscriptionStatus, AppError> {
    let active_subscription = get_user_active_subscription_with_plan(pool, user_id).await?;
    
    let (has_active_subscription, subscription_expires_at, days_remaining) = 
        if let Some(ref subscription) = active_subscription {
            let expires_at = subscription.end_date;
            let days_remaining = if let Some(end_date) = expires_at {
                let today = chrono::Utc::now().date_naive();
                let diff = end_date.signed_duration_since(today);
                Some(diff.num_days())
            } else {
                None
            };
            (true, expires_at, days_remaining)
        } else {
            (false, None, None)
        };

    Ok(SubscriptionStatus {
        has_active_subscription,
        current_subscription: active_subscription,
        subscription_expires_at,
        days_remaining,
    })
}