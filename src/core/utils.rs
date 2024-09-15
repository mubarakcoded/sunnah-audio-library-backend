use actix_web::web;
use redis::Commands;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;

use super::{AppError, AppErrorType};

pub fn set_to_cache<T>(
    redis_client: web::Data<redis::Client>,
    cache_key: &str,
    data: &T,
) -> Result<(), AppError>
where
    T: Serialize,
{
    let mut redis_conn = redis_client.get_connection().map_err(|e| AppError {
        message: Some(format!("Failed to connect to Redis: {}", e)),
        cause: Some(e.to_string()),
        error_type: AppErrorType::CacheError,
    })?;

    // Serialize the data to JSON
    let json_string = serde_json::to_string(data).map_err(|e| AppError {
        message: Some(format!("Failed to serialize data: {}", e)),
        cause: Some(e.to_string()),
        error_type: AppErrorType::JsonParseError,
    })?;

    // Cache the serialized data in Redis
    redis_conn
        .set(cache_key, json_string)
        .map_err(|e| AppError {
            message: Some(format!("Failed to cache data: {}", e)),
            cause: Some(e.to_string()),
            error_type: AppErrorType::CacheError,
        })?;

    Ok(())
}

pub fn get_from_cache<T>(
    redis_client: web::Data<redis::Client>,
    cache_key: &str,
) -> Result<Option<T>, AppError>
where
    T: DeserializeOwned,
{
    let mut redis_conn = redis_client.get_connection().map_err(|e| AppError {
        message: Some(format!("Failed to connect to Redis: {}", e)),
        cause: Some(e.to_string()),
        error_type: AppErrorType::CacheError,
    })?;

    if let Ok(cached_value) = redis_conn.get::<_, Option<String>>(cache_key) {
        if let Some(cached_value) = cached_value {
            let cached_data: T = serde_json::from_str(&cached_value).map_err(|e| AppError {
                message: Some(format!("Failed to parse cached data: {}", e)),
                cause: Some(e.to_string()),
                error_type: AppErrorType::JsonParseError,
            })?;
            return Ok(Some(cached_data));
        }
    }

    Ok(None)
}

// pub async fn get_default_vas_provider(
//     pool: &PgPool,
//     redis_client: web::Data<redis::Client>,
// ) -> Result<VASProvider, AppError> {
//     let cache_key = "default_vas_provider";

//     let mut redis_conn = redis_client.get_connection().map_err(|e| AppError {
//         message: Some(format!("Failed to connect to Redis: {}", e)),
//         cause: Some(e.to_string()),
//         error_type: AppErrorType::CacheError,
//     })?;

//     if let Ok(cached_value) = redis_conn.get::<_, Option<String>>(cache_key) {
//         if let Some(cached_value) = cached_value {
//             let provider: VASProvider =
//                 serde_json::from_str(&cached_value).map_err(|e| AppError {
//                     message: Some(format!("Failed to parse cached provider: {}", e)),
//                     cause: Some(e.to_string()),
//                     error_type: AppErrorType::JsonParseError,
//                 })?;
//             return Ok(provider);
//         }
//     }

//     let provider = vas_providers::get_default_provider(&pool).await?;

//     let json_string = serde_json::to_string(&provider).map_err(|e| AppError {
//         message: Some(format!("Failed to serialize provider: {}", e)),
//         cause: Some(e.to_string()),
//         error_type: AppErrorType::JsonParseError,
//     })?;

//     let _: () = redis_conn
//         .set(cache_key, json_string)
//         .map_err(|e| AppError {
//             message: Some(format!("Failed to cache provider: {}", e)),
//             cause: Some(e.to_string()),
//             error_type: AppErrorType::CacheError,
//         })?;

//     Ok(provider)
// }
