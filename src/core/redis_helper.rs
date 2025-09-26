use actix_web::web;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub struct RedisHelper {
    client: web::Data<redis::Client>,
}

#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNotFound,
}

impl RedisHelper {
    pub fn new(client: web::Data<redis::Client>) -> Self {
        Self { client }
    }

    async fn get_conn(&self) -> Result<redis::aio::Connection, RedisError> {
        self.client
            .get_async_connection()
            .await
            .map_err(RedisError::ConnectionError)
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, RedisError> {
        let mut conn = self.get_conn().await?;
        let value: Option<String> = conn.get(key).await?;
        match value {
            Some(v) => Ok(serde_json::from_str(&v)?),
            None => Err(RedisError::KeyNotFound),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        expiry: Option<Duration>,
    ) -> Result<(), RedisError> {
        let mut conn = self.get_conn().await?;
        let serialized = serde_json::to_string(value)?;
        match expiry {
            Some(exp) => conn.set_ex(key, serialized, exp.as_secs() as usize).await?,
            None => conn.set(key, serialized).await?,
        }
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.get_conn().await?;
        let deleted: i32 = conn.del(key).await?;
        Ok(deleted > 0)
    }

    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.get_conn().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn rpop<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let mut conn = self.get_conn().await?;
        let value: Option<String> = conn.rpop(key, None).await?;
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn lpop<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let mut conn = self.get_conn().await?;
        let value: Option<String> = conn.lpop(key, None).await?;
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn lpush<T: Serialize>(&self, key: &str, value: &T) -> Result<(), RedisError> {
        let mut conn = self.get_conn().await?;
        let serialized = serde_json::to_string(value)?;
        conn.lpush::<_, _, ()>(key, serialized).await?;
        Ok(())
    }

    // pub async fn lpush(&self, key: &str, value: &str) -> Result<(), RedisError> {
    //     let mut conn = self.get_conn().await?;
    //     conn.lpush(key, value).await?;
    //     Ok(())
    // }

    // pub async fn rpop(&self, key: &str, count: usize) -> Result<Option<String>, RedisError> {
    //     let mut conn = self.get_conn().await?;
    //     let result: Option<String> = conn
    //         .rpop(key, Some(NonZeroUsize::new(count).unwrap()))
    //         .await?;
    //     Ok(result)
    // }

    // pub async fn lpop(&self, key: &str) -> Result<Option<String>, RedisError> {
    //     let mut conn = self.get_conn().await?;
    //     let result: Option<String> = conn.lpop(key).await?;
    //     Ok(result)
    // }

    // pub async fn lrange(
    //     &self,
    //     key: &str,
    //     start: isize,
    //     stop: isize,
    // ) -> Result<Vec<String>, RedisError> {
    //     let mut conn = self.get_conn().await?;
    //     let result: Vec<String> = conn.lrange(key, start, stop).await?;
    //     Ok(result)
    // }
}
