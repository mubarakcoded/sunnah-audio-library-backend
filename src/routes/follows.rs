use crate::core::jwt_auth::JwtClaims;
use crate::core::AppError;
use crate::core::{AppErrorResponse, AppSuccessResponse};
use crate::db::follows;
use crate::models::follows::{FollowScholarRequest, UpdateFollowRequest};
use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use sqlx::MySqlPool;

#[tracing::instrument(name = "Follow Scholar", skip(pool, claims, request))]
#[post("/scholars/{scholar_id}/follow")]
pub async fn follow_scholar(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<FollowScholarRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let scholar_id = path.into_inner();
    
    // Validate that the scholar_id in path matches the request
    if scholar_id != request.scholar_id {
        return Ok(HttpResponse::BadRequest().json(AppErrorResponse {
            success: false,
            message: "Scholar ID in path doesn't match request body".to_string(),
        }));
    }

    let follow = follows::follow_scholar(&pool, user_id, &request).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: follow,
        message: "Scholar followed successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Unfollow Scholar", skip(pool, claims))]
#[delete("/scholars/{scholar_id}/follow")]
pub async fn unfollow_scholar(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let scholar_id = path.into_inner();
    follows::unfollow_scholar(&pool, user_id, scholar_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "Scholar unfollowed successfully"}),
        message: "Scholar unfollowed successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Update Follow Settings", skip(pool, claims, request))]
#[put("/scholars/{scholar_id}/follow")]
pub async fn update_follow_settings(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<UpdateFollowRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let scholar_id = path.into_inner();
    let follow = follows::update_follow_settings(&pool, user_id, scholar_id, &request).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: follow,
        message: "Follow settings updated successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get User Followed Scholars", skip(pool, claims))]
#[get("/my-follows")]
pub async fn get_my_followed_scholars(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let follows_list = follows::get_user_followed_scholars(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: follows_list,
        message: "Followed scholars retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Check Follow Status", skip(pool, claims))]
#[get("/scholars/{scholar_id}/follow-status")]
pub async fn check_follow_status(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let scholar_id = path.into_inner();
    let is_following = follows::is_following_scholar(&pool, user_id, scholar_id).await?;
    let followers_count = follows::get_scholar_followers_count(&pool, scholar_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({
            "is_following": is_following,
            "followers_count": followers_count
        }),
        message: "Follow status retrieved successfully".to_string(),
        pagination: None,
    }))
}