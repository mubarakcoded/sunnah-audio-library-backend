use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{error::ErrorUnauthorized, http, FromRequest, HttpMessage, HttpRequest};
use core::fmt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::core::AppError;

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
        //  let data = req.app_data::<web::Data<AppParameters>>().unwrap();

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|value| value.to_str().unwrap().split_at(7).1.to_string());

        if token.is_none() {
            let error = ErrorResponse {
                message: "Invalid login credentials".to_string(),
                success: false,
            };

            return ready(Err(ErrorUnauthorized(error)));
        }

        let claims = match decode::<JwtClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret("UDAFMIEOLANAOIEWOLADFWEALMOPNVALKAE".to_string().as_ref()),
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

const JWT_SECRET: &str = "UDAFMIEOLANAOIEWOLADFWEALMOPNVALKAE";

pub fn generate_jwt_token(claims: &JwtClaims) -> Result<String, AppError> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(JWT_SECRET.as_ref());
    
    encode(&header, claims, &encoding_key)
        .map_err(|_| AppError::internal_error("Failed to generate JWT token"))
}

impl FromRequest for JwtClaims {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(claims) = req.extensions().get::<JwtClaims>() {
            ready(Ok(claims.clone()))
        } else {
            let error = ErrorResponse {
                message: "No authentication token found".to_string(),
                success: false,
            };
            ready(Err(ErrorUnauthorized(error)))
        }
    }
}