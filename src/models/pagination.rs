use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

impl PaginationQuery {
    pub fn validate(&mut self) {
        if self.page < 1 {
            self.page = 1;
        }
        if self.per_page < 1 || self.per_page > 100 {
            self.per_page = 10;
        }
    }

    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub current_page: i32,
    pub per_page: i32,
    pub total_items: i64,
    pub total_pages: i32,
}

impl PaginationMeta {
    pub fn new(current_page: i32, per_page: i32, total_items: i64) -> Self {
        let total_pages = if total_items == 0 {
            1
        } else {
            (total_items as f64 / per_page as f64).ceil() as i32
        };

        Self {
            current_page,
            per_page,
            total_items,
            total_pages,
        }
    }
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    10
}
