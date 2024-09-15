use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub items_per_page: Option<i64>,
}