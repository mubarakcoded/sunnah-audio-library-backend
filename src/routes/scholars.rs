use crate::{
    core::{jwt_auth, AppError, AppErrorType, AppSuccessResponse},
    db::states, models::{books::Book, scholars::Scholar},
};
use actix_web::{
    get, post, web::{self}, HttpResponse, Responder
};

use crate::db::scholars;
use sqlx::MySqlPool;
use tracing::instrument;

#[instrument(name = "Get Scholars", skip(pool))]
#[get("")]
pub async fn get_scholars(
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, AppError> {
    let result = scholars::fetch_scholars(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholars: {:?}", e);
            AppError {
                message: Some("Failed to fetch scholars".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholars retrieved successfully".to_string(),
        data: Some(result),
    }))
}

#[instrument(name = "Get Scholars by State", skip(pool))]
#[get("/state/{state_id}")]
pub async fn get_scholars_by_state(
    pool: web::Data<MySqlPool>,
    state_id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let result = scholars::fetch_scholars_by_state(pool.get_ref(), state_id.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch scholars by state: {:?}", e);
            AppError {
                message: Some("Failed to fetch scholars by state".to_string()),
                cause: Some(e.to_string()),
                error_type: AppErrorType::InternalServerError,
            }
        })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "Scholars retrieved successfully".to_string(),
        data: Some(result),
    }))
}

// #[post("")]
// async fn create_scholar(
//     db_pool: web::Data<MySqlPool>,
//     scholar: web::Json<Scholar>,
// ) -> Result<impl Responder, AppError> {
//     sqlx::query!(
//         "INSERT INTO tbl_scholars (name, state, slug, about, image, status, created_by, created_at, updated_at) 
//         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
//         scholar.name,
//         scholar.state,
//         scholar.slug,
//         scholar.about,
//         scholar.image,
//         scholar.status,
//         scholar.created_by,
//         scholar.created_at,
//         scholar.updated_at
//     )
//     .execute(db_pool.get_ref())
//     .await
//     .map_err(|e| AppError::db_error(e.to_string()))?;

//     Ok(HttpResponse::Ok().json(AppSuccessResponse {
//         success: true,
//         message: "Scholar created successfully".to_string(),
//         data: None,
//     }))
// }

// #[get("/scholars/{id}/books")]
// async fn get_scholar_books(
//     db_pool: web::Data<MySqlPool>,
//     scholar_id: web::Path<i32>,
// ) -> Result<impl Responder, AppError> {
//     let books: Vec<Book> = sqlx::query_as!(
//         Book,
//         "SELECT id, scholar_id, name, slug, about, CONCAT('http://yourdomain.com/images/books/', image) as image, status, created_at, created_by, updated_at FROM tbl_books WHERE scholar_id = ? AND status = 'active'",
//         scholar_id.into_inner()
//     )
//     .fetch_all(db_pool.get_ref())
//     .await
//     .map_err(|e| AppError::db_error(e.to_string()))?;

//     Ok(HttpResponse::Ok().json(AppSuccessResponse {
//         success: true,
//         message: "Books retrieved successfully".to_string(),
//         data: Some(books),
//     }))
// }
