use chrono::Utc;
use sqlx::MySqlPool;
use std::time::Duration;
use tracing::{error, info};

/// Background job that checks for expired subscriptions and updates their status
pub async fn start_subscription_expiry_checker(pool: MySqlPool) {
    info!("Starting subscription expiry checker background job");
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Run every hour
        
        loop {
            interval.tick().await;
            
            if let Err(e) = check_and_expire_subscriptions(&pool).await {
                error!("Failed to check expired subscriptions: {}", e);
            }
        }
    });
}

/// Check and update expired subscriptions
async fn check_and_expire_subscriptions(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();
    let today = Utc::now().date_naive();
    
    // Update subscriptions that have passed their end_date
    let result = sqlx::query!(
        r#"
        UPDATE tbl_user_subscriptions
        SET status = 'expired', updated_at = ?
        WHERE status = 'active' 
        AND end_date IS NOT NULL 
        AND end_date < ?
        "#,
        now,
        today
    )
    .execute(pool)
    .await?;
    
    let rows_affected = result.rows_affected();
    
    if rows_affected > 0 {
        info!("Expired {} subscription(s)", rows_affected);
    }
    
    Ok(())
}

/// Manual function to expire subscriptions (can be called from an admin endpoint)
pub async fn expire_subscriptions_now(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let now = Utc::now().naive_utc();
    let today = Utc::now().date_naive();
    
    let result = sqlx::query!(
        r#"
        UPDATE tbl_user_subscriptions
        SET status = 'expired', updated_at = ?
        WHERE status = 'active' 
        AND end_date IS NOT NULL 
        AND end_date < ?
        "#,
        now,
        today
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected())
}
