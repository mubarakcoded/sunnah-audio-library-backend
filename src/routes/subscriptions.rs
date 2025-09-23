use crate::core::jwt_auth::JwtClaims;
use crate::core::AppError;
use crate::core::{AppErrorResponse, AppSuccessResponse};
use crate::db::subscriptions;
use crate::models::subscriptions::{CreateSubscriptionRequest, VerifySubscriptionRequest};

use actix_web::{get, post, put, web, HttpResponse, Result};
use sqlx::MySqlPool;

#[tracing::instrument(name = "Get Subscription Plans", skip(pool))]
#[get("/plans")]
pub async fn get_subscription_plans(
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, AppError> {
    let plans = subscriptions::get_all_subscription_plans(&pool).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: plans,
        message: "Subscription plans retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get User Subscriptions", skip(pool, claims))]
#[get("/my-subscriptions")]
pub async fn get_user_subscriptions(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let subscriptions = subscriptions::get_user_subscriptions(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: subscriptions,
        message: "User subscriptions retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Subscription Status", skip(pool, claims))]
#[get("/status")]
pub async fn get_subscription_status(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let status = subscriptions::get_user_subscription_status(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: status,
        message: "Subscription status retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Active Subscription", skip(pool, claims))]
#[get("/active")]
pub async fn get_active_subscription(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let active_subscription = subscriptions::get_user_active_subscription_with_plan(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: active_subscription,
        message: "Active subscription retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Create Subscription", skip(pool, claims, request))]
#[post("/subscribe")]
pub async fn create_subscription(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<CreateSubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    // Validate subscription plan exists
    let _plan = subscriptions::get_subscription_plan_by_id(&pool, request.subscription_plan_id).await
        .map_err(|_| AppError::bad_request("Invalid subscription plan ID"))?;

    // Check if user already has a pending subscription
    let user_subscriptions = subscriptions::get_user_subscriptions(&pool, user_id).await?;
    let has_pending = user_subscriptions.iter().any(|s| s.status == "pending");
    
    if has_pending {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "You already have a pending subscription. Please wait for verification.".to_string(),
        }));
    }

    let subscription = subscriptions::create_user_subscription(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: subscription,
        message: "Subscription created successfully. Please wait for admin verification.".to_string(),
        pagination: None,
    }))
}

// Admin endpoints
#[tracing::instrument(name = "Get Pending Subscriptions", skip(pool, claims))]
#[get("/admin/pending")]
pub async fn get_pending_subscriptions(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(AppErrorResponse {
            success: false,
            message: "Access denied. Admin role required.".to_string(),
        }));
    }

    let pending_subscriptions = subscriptions::get_pending_subscriptions(&pool).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: pending_subscriptions,
        message: "Pending subscriptions retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Verify Subscription", skip(pool, claims, request))]
#[put("/admin/verify/{subscription_id}")]
pub async fn verify_subscription(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<VerifySubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json(AppErrorResponse {
            success: false,
            message: "Access denied. Admin role required.".to_string(),
        }));
    }

    let subscription_id = path.into_inner();

    // Validate status
    if !["active", "cancelled"].contains(&request.status.as_str()) {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "Invalid status. Must be 'active' or 'cancelled'.".to_string(),
        }));
    }

    let subscription = subscriptions::verify_user_subscription(&pool, subscription_id, &request).await?;

    let message = match request.status.as_str() {
        "active" => "Subscription activated successfully",
        "cancelled" => "Subscription cancelled successfully",
        _ => "Subscription updated successfully",
    };

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: subscription,
        message: message.to_string(),
        pagination: None,
    }))
}