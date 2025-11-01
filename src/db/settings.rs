use crate::core::AppError;
use crate::models::settings::SiteSettings;
use sqlx::MySqlPool;

pub async fn fetch_site_settings(pool: &MySqlPool) -> Result<SiteSettings, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT 
            id,
            site_name,
            tagline,
            support_email,
            support_phone,
            address,
            facebook_url,
            twitter_url,
            instagram_url,
            youtube_url,
            whatsapp_number,
            additional_contacts,
            bank_name,
            account_name,
            account_number,
            created_at,
            updated_at
        FROM tbl_site_settings
        ORDER BY updated_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(SiteSettings {
        id: row.id as i32,
        site_name: row.site_name,
        tagline: row.tagline,
        support_email: row.support_email,
        support_phone: row.support_phone,
        address: row.address,
        facebook_url: row.facebook_url,
        twitter_url: row.twitter_url,
        instagram_url: row.instagram_url,
        youtube_url: row.youtube_url,
        whatsapp_number: row.whatsapp_number,
        additional_contacts: row.additional_contacts,
        bank_name: row.bank_name,
        account_name: row.account_name,
        account_number: row.account_number,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}


