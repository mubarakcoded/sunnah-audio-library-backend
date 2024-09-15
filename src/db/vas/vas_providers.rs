use sqlx::PgPool;
use sqlx::Transaction;
use tracing;
use uuid::Uuid;

use crate::core::{AppError, AppErrorType};
use crate::models::vas_providers::CreateVASProvider;
use crate::models::vas_providers::UpdateVASProvider;
use crate::models::vas_providers::VASProvider;

pub async fn create_vas_provider(
    pool: &PgPool,
    request: CreateVASProvider,
) -> Result<VASProvider, AppError> {
    let provider = sqlx::query_as!(
        VASProvider,
        r#"
        INSERT INTO vas_providers (provider_name, slug, service_code, logo_url)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        request.provider_name,
        request.slug,
        request.service_code,
        request.logo_url,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(provider)
}

pub async fn list_vas_providers(pool: &PgPool) -> Result<Vec<VASProvider>, AppError> {
    let providers = sqlx::query_as!(VASProvider, r#"SELECT * FROM vas_providers"#)
        .fetch_all(pool)
        .await
        .map_err(|e| map_sqlx_error(e))?;

    Ok(providers)
}

pub async fn get_vas_provider_by_id(
    pool: &PgPool,
    provider_id: Uuid,
) -> Result<VASProvider, AppError> {
    let provider = sqlx::query_as!(
        VASProvider,
        r#"SELECT * FROM vas_providers WHERE provider_id = $1"#,
        provider_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(provider)
}

pub async fn update_vas_provider(
    pool: &PgPool,
    provider_id: Uuid,
    request: UpdateVASProvider,
) -> Result<VASProvider, AppError> {
    let provider = sqlx::query_as!(
        VASProvider,
        r#"
        UPDATE vas_providers
        SET provider_name = COALESCE($1, provider_name),
            service_code = COALESCE($2, service_code),
            logo_url = COALESCE($3, logo_url),
            status = COALESCE($4, status),
            updated_at = NOW()
        WHERE provider_id = $5
        RETURNING *
        "#,
        request.provider_name,
        request.service_code,
        request.logo_url,
        request.status,
        provider_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| map_sqlx_error(e))?;

    Ok(provider)
}

pub async fn deactivate_vas_provider(pool: &PgPool, provider_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE vas_providers
        SET status = 'inactive', updated_at = NOW()
        WHERE provider_id = $1
        "#,
        provider_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_vas_provider(pool: &PgPool, provider_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM vas_providers WHERE provider_id = $1"#,
        provider_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_default_provider(pool: &PgPool) -> Result<VASProvider, AppError> {
    let provider = sqlx::query_as!(
        VASProvider,
        r#"SELECT * FROM vas_providers WHERE is_default = TRUE"#
    )
    .fetch_one(pool)
    .await?;

    Ok(provider)
}


pub async fn set_all_providers_not_default(
    tx: &mut Transaction<'_, sqlx::Postgres>,
) -> Result<(), AppError> {
    sqlx::query!("UPDATE vas_providers SET is_default = FALSE")
        .execute(tx.as_mut())
        .await?;
    Ok(())
}

pub async fn set_provider_default(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    provider_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE vas_providers SET is_default = TRUE WHERE provider_id = $1",
        provider_id
    )
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    tracing::error!("Database error: {:?}", err);
    AppError {
        message: Some(format!("Database error: {}", err)),
        cause: Some(err.to_string()),
        error_type: AppErrorType::DbError,
    }
}
