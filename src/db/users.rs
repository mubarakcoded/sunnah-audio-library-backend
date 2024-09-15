use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::AppError;

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct UserTbl {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


impl UserTbl {
    #[tracing::instrument(name = "Get user details by email", skip(db_pool))]
    pub async fn fetch_user_by_username(
        db_pool: &PgPool,
        username: &str,
    ) -> Result<Option<UserTbl>, AppError> {
        match sqlx::query_as::<_, UserTbl>("SELECT * FROM users where username = $1")
            .bind(&username)
            .fetch_optional(db_pool)
            .await
        {
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                Err(AppError::db_error(e))
            }
            Ok(x) => Ok(x),
        }
    }

    #[tracing::instrument(name = "Inserting new user into Database", skip(db_pool, username))]
    pub async fn insert_user(
        db_pool: &PgPool,
        user_id: Uuid,
        username: &str,
        password: &str,
        role: &str,
    ) -> Result<Uuid, AppError> {
        let insert: Result<(Uuid,), sqlx::Error> = sqlx::query_as(
            r#"
       INSERT INTO users (
        user_id,
        username,
        password,
        role
       ) VALUES ($1, $2, $3, $4) returning user_id
      "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(password)
        .bind(role)
        .fetch_one(db_pool)
        .await;

        match insert {
            Err(error) => {
                tracing::error!("Failed to execute query: {:?}", error);
                Err(AppError::db_error(error))
            }
            Ok(user) => Ok(user.0),
        }
    }
}
