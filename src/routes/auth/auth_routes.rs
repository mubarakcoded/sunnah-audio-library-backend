use crate::core::{jwt_auth, AppError};
use crate::core::AppErrorType::{AuthError, PayloadValidationError};
use crate::db::users::UserTbl;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::models::users::Claims;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateUserPayload {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    pub password: String,
    pub role: String,
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UserLoginPayload {
    #[validate(length(min = 1, message = "Username is required"))]
    username: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        length(min = 6, message = "Password must be at least 6 characters")
    )]
    password: String,
}

#[post("login")]
async fn login(
    request: web::Json<UserLoginPayload>,
    db_pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let query = UserTbl::fetch_user_by_username(&db_pool, &request.username)
        .await?;

    if let Some(user) = query {
        let hash = PasswordHash::new(&user.password).map_err(|e| {
            tracing::error!("Failed to generate password hash: {:?}", e);
            AppError {
                message: Some("error generating password hash".to_string()),
                cause: None,
                error_type: AuthError,
            }
        })?;
        let is_valid = Argon2::default()
            .verify_password(&request.password.as_bytes(), &hash)
            .map_or(false, |_| true);

        if !is_valid {
            return Err(AppError {
                message: Some("Invalid username or password".to_string()),
                cause: None,
                error_type: AuthError,
            });
        }

        let now = Utc::now();
        let iat: usize = now.timestamp() as usize;
        // let exp = (now + Duration::days(7)).timestamp() as usize;
        let exp = (chrono::Local::now() + chrono::Duration::days(7)).timestamp() as usize;

        let claims = Claims {
            sub: user.user_id,
            iat,
            exp,
            role: user.role,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("UDAFMIEOLANAOIEWOLADFWEALMOPNVALKAE".to_string().as_ref()),
        )
        .map_err(|e| {
            tracing::error!("Failed to generate token string: {:?}", e);
            AppError {
                message: Some("Failed to generate token string".to_string()),
                error_type: AuthError,
                cause: Some(e.to_string()),
            }
        })?;

        return Ok(HttpResponse::Ok().json(json!({
            "token": token,
            "expire_at": claims.exp,

        })));
    }

    Err(AppError {
        message: Some("Invalid username or password".to_string()),
        cause: None,
        error_type: AuthError,
    })
}

#[tracing::instrument(name = "Inserting User", skip(payload, db_pool))]
#[post("register")]
pub async fn register(
    payload: web::Json<CreateUserPayload>,
    db_pool: web::Data<PgPool>,
    _: jwt_auth::JwtMiddleware,
) -> Result<impl Responder, AppError> {
    match &payload.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(AppError {
                message: Some(e.to_string()),
                cause: None,
                error_type: PayloadValidationError,
            })
        }
    }

    let password = &payload.password;

    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError {
            message: Some("can not hash password".to_string()),
            cause: None,
            error_type: AuthError,
        })?;

    UserTbl::insert_user(
        &db_pool,
        Uuid::new_v4(),
        &payload.username,
        &hashed_password.to_string(),
        &payload.role.to_string(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
      "code":"200",
      "status":"Success",
      "Message":"User created successfully"
      }
    )))
}
