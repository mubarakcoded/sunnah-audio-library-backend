use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAccess {
    pub id: i32,
    pub scholar_id: i32,
    pub user_id: i32,
    pub created_by: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: i32,
    pub accessible_scholars: Vec<ScholarAccess>,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScholarAccess {
    pub scholar_id: i32,
    pub scholar_name: String,
    pub can_upload: bool,
    pub can_download: bool,
    pub can_manage: bool,
}

#[derive(Debug, Deserialize)]
pub struct GrantAccessRequest {
    pub user_id: i32,
    pub scholar_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct RevokeAccessRequest {
    pub user_id: i32,
    pub scholar_id: i32,
}