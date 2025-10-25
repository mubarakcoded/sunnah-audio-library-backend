use crate::core::jwt_auth::JwtClaims;
use crate::core::{AppConfig, AppError, AppSuccessResponse};
use crate::db::playlists;
use crate::models::playlists::{CreatePlaylistRequest, UpdatePlaylistRequest, AddToPlaylistRequest};
use crate::models::pagination::PaginationQuery;
use actix_web::{delete, get, post, put, web, HttpResponse, Result};
use sqlx::MySqlPool;

#[tracing::instrument(name = "Create Playlist", skip(pool, claims, request))]
#[post("")]
pub async fn create_playlist(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    request: web::Json<CreatePlaylistRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let playlist = playlists::create_playlist(&pool, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: playlist,
        message: "Playlist created successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get My Playlists", skip(pool, claims))]
#[get("")]
pub async fn get_my_playlists(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let playlists_list = playlists::get_user_playlists(&pool, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: playlists_list,
        message: "Playlists retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Public Playlists", skip(pool, pagination))]
#[get("/public")]
pub async fn get_public_playlists(
    pool: web::Data<MySqlPool>,
    pagination: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let mut pagination = pagination.into_inner();
    pagination.validate();
    let limit = pagination.per_page as i32;
    let offset = pagination.offset() as i32;

    let playlists_list = playlists::get_public_playlists(&pool, Some(limit), Some(offset)).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: playlists_list,
        message: "Public playlists retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Playlist", skip(pool))]
#[get("/{playlist_id}")]
pub async fn get_playlist(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let playlist_id = path.into_inner();
    let playlist = playlists::get_playlist_by_id(&pool, playlist_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: playlist,
        message: "Playlist retrieved successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Update Playlist", skip(pool, claims, request))]
#[put("/{playlist_id}")]
pub async fn update_playlist(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<UpdatePlaylistRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let playlist_id = path.into_inner();
    let playlist = playlists::update_playlist(&pool, playlist_id, user_id, &request).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: playlist,
        message: "Playlist updated successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Delete Playlist", skip(pool, claims))]
#[delete("/{playlist_id}")]
pub async fn delete_playlist(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let playlist_id = path.into_inner();
    playlists::delete_playlist(&pool, playlist_id, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "Playlist deleted successfully"}),
        message: "Playlist deleted successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Add File to Playlist", skip(pool, claims, request))]
#[post("/{playlist_id}/files")]
pub async fn add_file_to_playlist(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<i32>,
    request: web::Json<AddToPlaylistRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let playlist_id = path.into_inner();
    let playlist_file = playlists::add_file_to_playlist(&pool, playlist_id, user_id, &request).await?;

    Ok(HttpResponse::Created().json(AppSuccessResponse {
        success: true,
        data: playlist_file,
        message: "File added to playlist successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Remove File from Playlist", skip(pool, claims))]
#[delete("/{playlist_id}/files/{file_id}")]
pub async fn remove_file_from_playlist(
    pool: web::Data<MySqlPool>,
    claims: JwtClaims,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse, AppError> {
    let user_id: i32 = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

    let (playlist_id, file_id) = path.into_inner();
    playlists::remove_file_from_playlist(&pool, playlist_id, file_id, user_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: serde_json::json!({"message": "File removed from playlist successfully"}),
        message: "File removed from playlist successfully".to_string(),
        pagination: None,
    }))
}

#[tracing::instrument(name = "Get Playlist Files", skip(pool))]
#[get("/{playlist_id}/files")]
pub async fn get_playlist_files(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let playlist_id = path.into_inner();
    let files = playlists::get_playlist_files(&pool, &config, playlist_id).await?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        data: files,
        message: "Playlist files retrieved successfully".to_string(),
        pagination: None,
    }))
}