use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: i32,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub content_type: String,
}

#[derive(Debug, Deserialize)]
pub struct FileUploadRequest {
    pub book_id: i32,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileDownloadInfo {
    pub file_id: i32,
    pub filename: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size: i64,
    pub book_id: i32,
    pub scholar_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct FileMetadata {
    pub title: String,
    pub description: Option<String>,
    pub duration: Option<i32>, // in seconds for audio files
    pub file_size: i64,
    pub content_type: String,
}