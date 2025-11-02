use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{error::ErrorUnauthorized, http, FromRequest, HttpMessage, HttpRequest};
use core::fmt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::core::{AppConfig, AppError};

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String, // user ID
    pub email: String,
    pub role: String,
    pub exp: usize, // expiration time
}

#[derive(Debug)]
pub struct JwtMiddleware {
    pub user_id: i32,
    pub claims: JwtClaims,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Get AppConfig from request data
        let config = match req.app_data::<actix_web::web::Data<AppConfig>>() {
            Some(cfg) => cfg.get_ref().clone(),
            None => {
                let error = ErrorResponse {
                    message: "Server configuration error".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(auth_header[7..].to_string())
                } else {
                    None
                }
            });

        if token.is_none() {
            let error = ErrorResponse {
                message: "No authentication token found".to_string(),
                success: false,
            };

            return ready(Err(ErrorUnauthorized(error)));
        }

        let claims = match decode::<JwtClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(config.get_jwt_secret().as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_ea) => {
                let error = ErrorResponse {
                    message: "Invalid token".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        let user_id: i32 = match claims.sub.parse() {
            Ok(id) => id,
            Err(_) => {
                let error = ErrorResponse {
                    message: "Invalid user ID in token".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        req.extensions_mut().insert(claims.clone());

        ready(Ok(JwtMiddleware { user_id, claims }))
    }
}

pub fn generate_jwt_token(claims: &JwtClaims, config: &AppConfig) -> Result<String, AppError> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(config.get_jwt_secret().as_ref());
    
    encode(&header, claims, &encoding_key)
        .map_err(|_| AppError::internal_error("Failed to generate JWT token"))
}

impl FromRequest for JwtClaims {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // First check if claims are already in extensions (from middleware)
        if let Some(claims) = req.extensions().get::<JwtClaims>() {
            return ready(Ok(claims.clone()));
        }

        // Get AppConfig from request data
        let config = match req.app_data::<actix_web::web::Data<AppConfig>>() {
            Some(cfg) => cfg.get_ref().clone(),
            None => {
                let error = ErrorResponse {
                    message: "Server configuration error".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        // If not in extensions, parse the token directly
        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(auth_header[7..].to_string())
                } else {
                    None
                }
            });

        if token.is_none() {
            let error = ErrorResponse {
                message: "No authentication token found".to_string(),
                success: false,
            };
            return ready(Err(ErrorUnauthorized(error)));
        }

        let claims = match decode::<JwtClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(config.get_jwt_secret().as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let error = ErrorResponse {
                    message: "Invalid token".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        ready(Ok(claims))
    }
}