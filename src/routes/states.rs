use actix_web::{
    get,
    web::{self},
    HttpResponse, Responder,
};
use sqlx::MySqlPool;
use tracing::instrument;

// const BANK_CODE: &str = "SHUNKU";
// const BANK_NAME: &str = "009291";

use crate::{
    core::{AppError, AppErrorType, AppSuccessResponse},
    db::states,
};

// #[tracing::instrument(name = "Get All Transactions", skip(db_pool, query), fields(
//     start_date = ?query.start_date,
//     end_date = ?query.end_date,
//     status = ?query.status,
//     transaction_reference = ?query.transaction_reference,
//     transaction_category = ?query.transaction_category,
//     transaction_type = ?query.transaction_type,
//     page = ?query.page,
//     page_size = ?query.page_size
// ))]
// #[get("transactions")]
// async fn get_all_transactions(
//     db_pool: web::Data<PgPool>,
//     query: web::Query<GetAllTransactionsQuery>,
// ) -> Result<impl Responder, AppError> {
//     let filters = query.into_inner();

//     let result = TransactionsTbl::get_all_transactions(
//         &db_pool,
//         filters.start_date,
//         filters.end_date,
//         filters.status,
//         filters.transaction_reference,
//         filters.transaction_type,
//         filters.transaction_category,
//         filters.page,
//         filters.page_size,
//     )
//     .await
//     .map_err(|e| {
//         tracing::error!("Failed to fetch transactions: {:?}", e);
//         AppError {
//             message: Some("Failed to fetch transactions".to_string()),
//             cause: Some(e.to_string()),
//             error_type: AppErrorType::InternalServerError,
//         }
//     })?;

//     Ok(HttpResponse::Ok().json(AppSuccessResponse {
//         success: true,
//         message: "Transactions retrieved successfully".to_string(),
//         data: Some(result),
//     }))
// }

// #[tracing::instrument(name = "Get Transaction by ID", skip(db_pool), fields(transaction_id = %path))]
// #[get("/transactions/{id}")]
// async fn get_transaction_by_id(
//     db_pool: web::Data<PgPool>,
//     path: web::Path<Uuid>,
// ) -> Result<impl Responder, AppError> {
//     let transaction_id = path.into_inner();
//     let result = TransactionsTbl::get_transaction_by_id(&db_pool, transaction_id)
//         .await
//         .map_err(|e| {
//             tracing::error!(
//                 "Failed to fetch transaction with id {}: {:?}",
//                 transaction_id,
//                 e
//             );
//             AppError {
//                 message: Some(format!(
//                     "Failed to fetch transaction with id {}",
//                     transaction_id
//                 )),
//                 cause: Some(e.to_string()),
//                 error_type: AppErrorType::NotFoundError,
//             }
//         })?;

//     Ok(HttpResponse::Ok().json(AppSuccessResponse {
//         success: true,
//         message: "Transaction retrieved successfully".to_string(),
//         data: Some(result),
//     }))
// }

// // #[tracing::instrument(name = "Perform Transaction", skip(db_pool))]
// // #[post("/admin/transactions")]
// // async fn perform_transaction(
// //     db_pool: web::Data<PgPool>,
// //     transaction: web::Json<Transaction>,
// // ) -> Result<impl Responder, AppError> {
// //     // Implementation here
// // }

// #[tracing::instrument(name = "Get Transaction Metrics", skip(db_pool))]
// #[get("/transactions/metrics")]
// async fn get_transaction_metrics(db_pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
//     let result = TransactionsTbl::get_transaction_metrics(&db_pool)
//         .await
//         .map_err(|e| {
//             tracing::error!("Failed to fetch transaction metrics: {:?}", e);
//             AppError {
//                 message: Some("Failed to fetch transaction metrics".to_string()),
//                 cause: Some(e.to_string()),
//                 error_type: AppErrorType::InternalServerError,
//             }
//         })?;

//     Ok(HttpResponse::Ok().json(AppSuccessResponse {
//         success: true,
//         message: "Transaction metrics retrieved successfully".to_string(),
//         data: Some(result),
//     }))
// }

#[instrument(name = "Get States", skip(pool))]
#[get("/states")]
pub async fn get_states(pool: web::Data<MySqlPool>) -> Result<impl Responder, AppError> {
    let result = states::fetch_states(pool.get_ref()).await.map_err(|e| {
        tracing::error!("Failed to fetch states: {:?}", e);
        AppError {
            message: Some("Failed to fetch states".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::InternalServerError,
        }
    })?;

    Ok(HttpResponse::Ok().json(AppSuccessResponse {
        success: true,
        message: "States retrieved successfully".to_string(),
        data: Some(result),
        pagination: None,
    }))
}
