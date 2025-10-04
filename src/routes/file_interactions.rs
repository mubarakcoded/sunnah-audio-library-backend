use crate::core::jwt_auth::JwtClaims;
use crate::core::AppError;
use crate::core::AppSuccessResponse;
use crate::db::file_interactions;
use crate::models::file_interactions::{
    CreateReportRequest, ResolveReportRequest, LikeFileRequest,
    CreateCommentRequest, UpdateCommentRequest
};
use crate::models::pagination::PaginationQuery;
use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use sqlx::MySqlPool;

// File Reports
#[tracing::instrument(name = "Report File", skip(pool, claims, request))]
#[post("/reports")]
pub async fn report_file(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<CreateReportRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let report = file_interactions::create_file_report(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: report,
        message: "File reported successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Pending Reports", skip(pool, claims, pagination))]
#[get("/admin/reports/pending")]
pub async fn get_pending_reports(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    pagination: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if claims.role != "admin" && claims.role != "manager" {
        return Err(AppError::forbidden_error("Access denied"));
    }

    let mut pagination = pagination.into_inner();
    pagination.validate();
    let limit = pagination.per_page as i32;
    let offset = pagination.offset() as i32;

    let reports = file_interactions::get_pending_reports(&pool, Some(limit), Some(offset)).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: reports,
        message: "Pending reports retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Resolve Report", skip(pool, claims, request))]
#[put("/admin/reports/{report_id}/resolve")]
pub async fn resolve_report(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<ResolveReportRequest>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if claims.role != "admin" && claims.role != "manager" {
        return Err(AppError::forbidden_error("Access denied"));
    }

    let admin_user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let report_id = path.into_inner();
    let report = file_interactions::resolve_file_report(&pool, report_id, admin_user_id, &request).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: report,
        message: "Report resolved successfully".to_string(),
        pagination: None,
    }))
}

// File Likes
#[tracing::instrument(name = "Like File", skip(pool, claims, request))]
#[post("/likes")]
pub async fn like_file(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<LikeFileRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let like = file_interactions::like_file(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: like,
        message: "File liked successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Unlike File", skip(pool, claims))]
#[delete("/{file_id}/likes")]
pub async fn unlike_file(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let file_id = path.into_inner();
    file_interactions::unlike_file(&pool, user_id, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "File unliked successfully"}),
        message: "File unliked successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get File Likes", skip(pool))]
#[get("/{file_id}/likes")]
pub async fn get_file_likes(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let file_id = path.into_inner();
    let likes_count = file_interactions::get_file_likes_count(&pool, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({
            "file_id": file_id,
            "likes_count": likes_count
        }),
        message: "File likes retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Check File Like Status", skip(pool, claims))]
#[get("/{file_id}/like-status")]
pub async fn check_file_like_status(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let file_id = path.into_inner();
    let is_liked = file_interactions::is_file_liked_by_user(&pool, user_id, file_id).await?;
    let likes_count = file_interactions::get_file_likes_count(&pool, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({
            "file_id": file_id,
            "is_liked": is_liked,
            "likes_count": likes_count
        }),
        message: "File like status retrieved successfully".to_string(),
        pagination: None,
    }))
}

// File Comments
#[tracing::instrument(name = "Create Comment", skip(pool, claims, request))]
#[post("/comments")]
pub async fn create_comment(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<CreateCommentRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let comment = file_interactions::create_file_comment(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: comment,
        message: "Comment created successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get File Comments", skip(pool))]
#[get("/{file_id}/comments")]
pub async fn get_file_comments(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let file_id = path.into_inner();
    let comments = file_interactions::get_file_comments(&pool, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: comments,
        message: "File comments retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Update Comment", skip(pool, claims, request))]
#[put("/comments/{comment_id}")]
pub async fn update_comment(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<UpdateCommentRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let comment_id = path.into_inner();
    let comment = file_interactions::update_file_comment(&pool, comment_id, user_id, &request).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: comment,
        message: "Comment updated successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Delete Comment", skip(pool, claims))]
#[delete("/comments/{comment_id}")]
pub async fn delete_comment(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let comment_id = path.into_inner();
    file_interactions::delete_file_comment(&pool, comment_id, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "Comment deleted successfully"}),
        message: "Comment deleted successfully".to_string(),
        pagination: None,
    }))
}

// Download Stats
#[tracing::instrument(name = "Get File Download Stats", skip(pool))]
#[get("/{file_id}/download-stats")]
pub async fn get_file_download_stats(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let file_id = path.into_inner();
    let stats = file_interactions::get_file_download_stats(&pool, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: stats,
        message: "File download stats retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get User Download History", skip(pool, claims, pagination))]
#[get("/my-downloads")]
pub async fn get_my_download_history(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    pagination: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let mut pagination = pagination.into_inner();
    pagination.validate();
    let limit = pagination.per_page as i32;
    let offset = pagination.offset() as i32;

    let downloads = file_interactions::get_user_download_history(&pool, user_id, Some(limit), Some(offset)).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: downloads,
        message: "Download history retrieved successfully".to_string(),
        pagination: None,
    }))
}