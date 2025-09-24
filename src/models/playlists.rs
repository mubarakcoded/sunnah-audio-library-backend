use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub cover_image: Option<String>,
    pub total_files: i32,
    pub total_duration: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistFile {
    pub id: i32,
    pub playlist_id: i32,
    pub file_id: i32,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub cover_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub cover_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddToPlaylistRequest {
    pub file_id: i32,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderPlaylistRequest {
    pub file_orders: Vec<FileOrder>,
}

#[derive(Debug, Deserialize)]
pub struct FileOrder {
    pub file_id: i32,
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
pub struct PlaylistResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub cover_image: Option<String>,
    pub total_files: i32,
    pub total_duration: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub owner_name: String,
}

#[derive(Debug, Serialize)]
pub struct PlaylistFileResponse {
    pub file_id: i32,
    pub file_title: String,
    pub scholar_name: Option<String>,
    pub duration: String,
    pub sort_order: i32,
    pub added_at: NaiveDateTime,
}