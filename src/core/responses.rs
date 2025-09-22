use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::Error;
use redis::RedisError;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum AppErrorType {
    NotFoundError,
    DbError,
    AuthError,
    JsonDeserializationError,
    JsonSerializationError,
    JsonParseError,
    PayloadValidationError,
    ApiError { code: String, message: String },
    NetworkError,
    CacheError,
    InternalServerError,
    SerializationError,
    ForbiddenError,
    PinNotFound,
    HashingFailed,
    IncorrectPin,
    DefaultPin,
}

#[derive(Debug, PartialEq)]
pub struct AppError {
    pub error_type: AppErrorType,
    pub message: Option<String>,
    pub cause: Option<String>,
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub success: bool,
    pub message: String,
}

impl AppError {
    pub fn message(&self) -> String {
        match &*self {
            AppError {
                message: Some(message),
                ..
            } => message.clone(),

            AppError {
                message: None,
                error_type: AppErrorType::NotFoundError,
                ..
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occurred".to_string(),
        }
    }

    pub fn db_error(error: impl ToString) -> AppError {
        AppError {
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
            message: Some(error.to_string()),
        }
    }

    pub fn forbidden_error(error: impl ToString) -> AppError {
        AppError {
            cause: Some(error.to_string()),
            error_type: AppErrorType::ForbiddenError,
            message: Some(error.to_string()),
        }
    }

    pub fn unauthorized(error: impl ToString) -> AppError {
        AppError {
            cause: Some(error.to_string()),
            error_type: AppErrorType::AuthError,
            message: Some(error.to_string()),
        }
    }

    pub fn internal_error(error: impl ToString) -> AppError {
        AppError {
            cause: Some(error.to_string()),
            error_type: AppErrorType::InternalServerError,
            message: Some(error.to_string()),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: Error) -> Self {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError {
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
            message: Some(error.to_string()),
        }
    }
}

impl From<RedisError> for AppError {
    fn from(error: RedisError) -> Self {
        AppError {
            cause: Some(error.to_string()),
            message: Some("Internal Caching Error".to_string()),
            error_type: AppErrorType::CacheError,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(error: bcrypt::BcryptError) -> Self {
        AppError {
            error_type: AppErrorType::HashingFailed,
            message: Some(format!("Hashing failed: {}", error)),
            cause: None,
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::AuthError => StatusCode::UNAUTHORIZED,
            AppErrorType::DbError
            | AppErrorType::JsonParseError
            | AppErrorType::NetworkError
            | AppErrorType::CacheError
            | AppErrorType::SerializationError
            | AppErrorType::JsonDeserializationError
            | AppErrorType::JsonSerializationError
            | AppErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
            AppErrorType::PayloadValidationError => StatusCode::BAD_REQUEST,
            AppErrorType::ApiError { .. } => StatusCode::BAD_GATEWAY,
            AppErrorType::ForbiddenError => StatusCode::FORBIDDEN,
            AppErrorType::PinNotFound => StatusCode::NOT_FOUND,
            AppErrorType::HashingFailed => StatusCode::BAD_GATEWAY,
            AppErrorType::IncorrectPin => StatusCode::FORBIDDEN,
            AppErrorType::DefaultPin => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            success: false,
            message: self.message(),
        })
    }
}

#[derive(Serialize)]
pub struct AppSuccessResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<crate::models::pagination::PaginationMeta>,
}
