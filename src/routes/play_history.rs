use crate::core::jwt_auth::JwtClaims;
use crate::core::AppError;
use crate::core::{AppErrorResponse, AppSuccessResponse};
use crate::db::play_history;
use crate::models::pagination::PaginationInfo;
use crate::models::play_history::RecordPlayRequest;
use actix_web::{delete, get, post, web, HttpResponse, Result};
use sqlx::MySqlPool;

#[tracing::instrument(name = "Record Play History", skip(pool, claims, request))]
#[post("/play-history")]
pub async fn record_play(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<RecordPlayRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let play_record = play_history::record_play(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: play_record,
        message: "Play history recorded successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get User Play History", skip(pool, claims, query))]
#[get("/play-history")]
pub async fn get_my_play_history(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    query: web::Query<crate::models::pagination::PaginationParams>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let history = play_history::get_user_play_history(&pool, user_id, Some(limit), Some(offset)).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: history,
        message: "Play history retrieved successfully".to_string(),
        pagination: None,
    }))

}

#[tracing::instrument(name = "Get Most Played Files", skip(pool, claims, query))]
#[get("/play-history/most-played")]
pub async fn get_most_played_files(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let limit = query.get("limit")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    let most_played = play_history::get_user_most_played_files(&pool, user_id, limit).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: most_played,
        message: "Most played files retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Clear Play History", skip(pool, claims))]
#[delete("/play-history")]
pub async fn clear_play_history(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    play_history::clear_user_play_history(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "Play history cleared successfully"}),
        message: "Play history cleared successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get File Play Stats", skip(pool))]
#[get("/files/{file_id}/play-stats")]
pub async fn get_file_play_stats(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let file_id = path.into_inner();
    let (total_plays, unique_listeners) = play_history::get_file_play_stats(&pool, file_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({
            "file_id": file_id,
            "total_plays": total_plays,
            "unique_listeners": unique_listeners
        }),
        message: "File play stats retrieved successfully".to_string(),
        pagination: None,
    }))
}