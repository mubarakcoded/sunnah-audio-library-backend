use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use tracing::instrument;


use crate::{
    core::{jwt_auth::JwtMiddleware, AppError, AppErrorType, AppSuccessResponse},
    db::access,
    models::access::{GrantAccessRequest, RevokeAccessRequest},
};

#[instrument(name = "Get User Permissions", skip(pool))]
#[get("/permissions")]
pub async fn get_user_permissions(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
) -> Result<impl Responder, AppError> {
    let permissions = access::fetch_user_permissions(pool.get_ref(), auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user permissions: {:?}", e);
            AppError {
                message: Some("Failed to fetch user permissions".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "User permissions retrieved successfully".to_string(),
        data: Some(permissions),
        pagination: None,
    }))
}

#[instrument(name = "Grant User Access", skip(pool))]
#[post("/access/grant")]
pub async fn grant_access(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    request: web::Json<GrantAccessRequest>,
) -> Result<impl Responder, AppError> {
    // Only admins and managers can grant access
    let user_permissions = access::fetch_user_permissions(pool.get_ref(), auth.user_id).await?;
    
    if !matches!(user_permissions.role.as_str(), "Admin" | "Manager") {
        return Err(AppError {
            message: Some("Insufficient permissions to grant access".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    access::grant_user_access(
        pool.get_ref(),
        request.user_id,
        request.scholar_id,
        auth.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to grant access: {:?}", e);
        AppError {
            message: Some("Failed to grant access".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Access granted successfully".to_string(),
        data: Some(()),
        pagination: None,
    }))
}

#[instrument(name = "Revoke User Access", skip(pool))]
#[post("/access/revoke")]
pub async fn revoke_access(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
    request: web::Json<RevokeAccessRequest>,
) -> Result<impl Responder, AppError> {
    // Only admins and managers can revoke access
    let user_permissions = access::fetch_user_permissions(pool.get_ref(), auth.user_id).await?;
    
    if !matches!(user_permissions.role.as_str(), "Admin" | "Manager") {
        return Err(AppError {
            message: Some("Insufficient permissions to revoke access".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    access::revoke_user_access(pool.get_ref(), request.user_id, request.scholar_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke access: {:?}", e);
            AppError {
                message: Some("Failed to revoke access".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Access revoked successfully".to_string(),
        data: Some(()),
        pagination: None,
    }))
}

#[instrument(name = "Get All User Accesses", skip(pool))]
#[get("/access/all")]
pub async fn get_all_accesses(
    pool: web::Data<MySqlPool>,
    auth: JwtMiddleware,
) -> Result<impl Responder, AppError> {
    // Only admins can view all accesses
    let user_permissions = access::fetch_user_permissions(pool.get_ref(), auth.user_id).await?;
    
    if user_permissions.role != "Admin" {
        return Err(AppError {
            message: Some("Insufficient permissions to view all accesses".to_string()),
            cause: None,
            error_type: AppErrorType::ForbiddenError,
        });
    }

    let accesses = access::fetch_all_user_accesses(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch all accesses: {:?}", e);
            AppError {
                message: Some("Failed to fetch all accesses".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "All accesses retrieved successfully".to_string(),
        data: Some(accesses),
        pagination: None,
    }))
}