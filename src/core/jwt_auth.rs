use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{error::ErrorUnauthorized, http, FromRequest, HttpMessage, HttpRequest};
use core::fmt;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::models::users::Claims;

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

pub struct JwtMiddleware {
    pub user_id: Uuid,
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

        let claims = match decode::<Claims>(
            &token.unwrap(),
            &DecodingKey::from_secret("UDAFMIEOLANAOIEWOLADFWEALMOPNVALKAE".to_string().as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(ea) => {
                let error = ErrorResponse {
                    message: "Invalid token".to_string(),
                    success: false,
                };
                return ready(Err(ErrorUnauthorized(error)));
            }
        };

        let user_id = claims.sub;
        req.extensions_mut().insert(user_id.clone());

        ready(Ok(JwtMiddleware { user_id }))
    }
}
