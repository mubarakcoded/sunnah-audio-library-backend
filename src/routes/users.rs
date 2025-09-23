use crate::core::jwt_auth::{generate_jwt_token, JwtClaims};
use crate::core::AppError;
use crate::core::{AppErrorResponse, AppSuccessResponse};
use crate::core::redis_helper::RedisHelper;
use crate::core::EmailService;
use crate::db::users;
use crate::models::users::{
    ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, MessageResponse,
    RegisterRequest, ResetPasswordRequest, UpdateProfileRequest, UserProfile, OtpData,
};
use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use rand::Rng;
use std::time::Duration as StdDuration;

#[tracing::instrument(name = "Register User", skip(pool, request))]
#[post("/register")]
pub async fn register(
    pool: web::Data<MySqlPool>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    // Check if email already exists
    if users::email_exists(&pool, &request.email).await? {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "A user with this email address already exists".to_string(),
        }));
    }

    // Validate email format
    if !is_valid_email(&request.email) {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "Please provide a valid email address".to_string(),
        }));
    }

    // Validate password strength
    if request.password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "Password must be at least 6 characters long".to_string(),
        }));
    }

    let user = users::create_user(&pool, &request).await?;
    let user_profile = UserProfile::from(user);

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: user_profile,
        message: "User registered successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "User Login", skip(pool, request))]
#[post("/login")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user by email
    let user = match users::get_user_by_email(&pool, &request.email).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(AppErrorResponse {
                success: false,
                message: "Email or password is incorrect".to_string(),
            }));
        }
    };

    // Verify password
    if !users::verify_password(&request.password, &user.password).await? {
        return Ok(HttpResponse::Unauthorized().json(AppErrorResponse {
            success: false,
            message: "Email or password is incorrect".to_string(),
        }));
    }

    // Generate JWT token
    let expires_at = Utc::now() + Duration::hours(24);
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: expires_at.timestamp() as usize,
    };

    let token = generate_jwt_token(&claims)?;
    let user_profile = UserProfile::from(user.clone());

    // Get user subscription status
    let subscription_status = match crate::db::subscriptions::get_user_subscription_status(&pool, user.id).await {
        Ok(status) => Some(status),
        Err(_) => None, // Don't fail login if subscription check fails
    };

    let response = LoginResponse {
        user: user_profile,
        token,
        expires_at,
        subscription_status,
    };

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: response,
        message: "Login successful".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get User Profile", skip(pool, claims))]
#[get("/profile")]
pub async fn get_profile(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let user = users::get_user_by_id(&pool, user_id).await?;
    let user_profile = UserProfile::from(user);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: user_profile,
        message: "Profile retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Update User Profile", skip(pool, claims, request))]
#[put("/profile")]
pub async fn update_profile(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let user = users::update_user_profile(&pool, user_id, &request).await?;
    let user_profile = UserProfile::from(user);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: user_profile,
        message: "Profile updated successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Change User Password", skip(pool, claims, request))]
#[post("/change-password")]
pub async fn change_password(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    // Get current user to verify current password
    let user = users::get_user_by_id(&pool, user_id).await?;

    // Verify current password
    if !users::verify_password(&request.current_password, &user.password).await? {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "The current password you provided is incorrect".to_string(),
        }));
    }

    // Validate new password strength
    if request.new_password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "New password must be at least 6 characters long".to_string(),
        }));
    }

    users::change_user_password(&pool, user_id, &request.new_password).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: MessageResponse {
            message: "Password changed successfully".to_string(),
        },
        message: "Password changed successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Forgot Password", skip(pool, request, redis_service, email_service))]
#[post("/forgot-password")]
pub async fn forgot_password(
    pool: web::Data<MySqlPool>,
    redis_service: web::Data<RedisHelper>,
    email_service: web::Data<EmailService>,
    request: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    // Check if user exists
    let user = match users::get_user_by_email(&pool, &request.email).await {
        Ok(user) => user,
        Err(_) => {
            // Don't reveal if email exists or not for security
            // Still generate and "send" OTP to prevent timing attacks
            let _dummy_otp = generate_otp();
            let _ = send_otp_email(&email_service, &request.email, &_dummy_otp).await;
            
            return Ok(HttpResponse::Ok().json(AppSuccessResponse {
                success: true,
                data: MessageResponse {
                    message: "If the email exists, an OTP has been sent to your email address".to_string(),
                },
                message: "Password reset request processed".to_string(),
                pagination: None,
            }));
        }
    };

    // Generate OTP
    let otp = generate_otp();
    let otp_data = OtpData {
        email: user.email.clone(),
        otp: otp.clone(),
        created_at: Utc::now().timestamp(),
    };

    // Store OTP in Redis with 10 minutes expiration
    let redis_key = get_otp_redis_key(&user.email);
    let expiry = StdDuration::from_secs(10 * 60); // 10 minutes
    
    redis_service.set(&redis_key, &otp_data, Some(expiry)).await
        .map_err(|e| AppError::internal_error(format!("Failed to store OTP: {}", e)))?;

    // Send OTP via email
    send_otp_email(&email_service, &user.email, &otp).await?;

    tracing::info!("Password reset OTP generated for user: {}", user.email);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: MessageResponse {
            message: "An OTP has been sent to your email address. Please use it to reset your password within 10 minutes.".to_string(),
        },
        message: "Password reset OTP sent successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Reset Password", skip(pool, redis_service, email_service, request))]
#[post("/reset-password")]
pub async fn reset_password(
    pool: web::Data<MySqlPool>,
    redis_service: web::Data<RedisHelper>,
    email_service: web::Data<EmailService>,
    request: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate new password strength
    if request.new_password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "New password must be at least 6 characters long".to_string(),
        }));
    }

    // Check if user exists
    let user = match users::get_user_by_email(&pool, &request.email).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
                success: false,
                message: "Invalid email or OTP".to_string(),
            }));
        }
    };

    // Get OTP from Redis
    let redis_key = get_otp_redis_key(&request.email);
    
    let stored_otp_data: OtpData = match redis_service.get(&redis_key).await {
        Ok(data) => data,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
                success: false,
                message: "Invalid or expired OTP. Please request a new one.".to_string(),
            }));
        }
    };

    // Validate OTP
    if stored_otp_data.otp != request.otp || stored_otp_data.email != request.email {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "Invalid OTP".to_string(),
        }));
    }

    // Check if OTP is expired (additional check, Redis should handle expiry)
    let current_time = Utc::now().timestamp();
    let otp_age = current_time - stored_otp_data.created_at;
    if otp_age > 600 { // 10 minutes in seconds
        // Delete expired OTP
        let _ = redis_service.delete(&redis_key).await;
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "OTP has expired. Please request a new one.".to_string(),
        }));
    }

    // Reset password
    users::change_user_password(&pool, user.id, &request.new_password).await?;

    // Delete used OTP from Redis
    let _ = redis_service.delete(&redis_key).await;

    // Send password reset confirmation email
    if let Err(e) = email_service.send_password_reset_confirmation(&user.email).await {
        tracing::warn!("Failed to send password reset confirmation email: {}", e);
        // Don't fail the request if confirmation email fails
    }

    tracing::info!("Password reset successful for user: {}", user.email);

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: MessageResponse {
            message: "Password reset successfully. You can now login with your new password.".to_string(),
        },
        message: "Password reset successful".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Deactivate User Account", skip(pool, claims))]
#[delete("/deactivate")]
pub async fn deactivate_account(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    users::deactivate_user(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: MessageResponse {
            message: "Account deactivated successfully".to_string(),
        },
        message: "Account deactivated successfully".to_string(),
        pagination: None,
    }))
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

async fn send_otp_email(email_service: &crate::core::EmailService, email: &str, otp: &str) -> Result<(), AppError> {
    // Send OTP via email using SMTP
    email_service.send_otp_email(email, otp).await?;
    
    // Also log for development/debugging (remove in production if needed)
    tracing::info!("OTP sent to email: {} (OTP: {} - remove this log in production)", email, otp);
    
    Ok(())
}

fn get_otp_redis_key(email: &str) -> String {
    format!("password_reset_otp:{}", email)
}
