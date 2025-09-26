use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayHistory {
    pub id: i32,
    pub user_id: i32,
    pub file_id: i32,
    pub played_duration: i32,
    pub total_duration: Option<i32>,
    pub play_position: Option<i32>,
    pub play_action: String,
    pub device_type: Option<String>,
    pub played_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RecordPlayRequest {
    pub file_id: i32,
    pub played_duration: i32,            // Duration played in seconds
    pub total_duration: Option<i32>,     // Total file duration in seconds
    pub play_position: Option<i32>,      // Current position when paused/stopped
    pub play_action: PlayAction,         // What triggered this call
    pub device_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum PlayAction {
    Start,      // User clicked play
    Pause,      // User paused
    Resume,     // User resumed
    Complete,   // File finished playing
    Skip,       // User skipped to next/previous
    Stop,       // User stopped playback
    Progress,   // Periodic progress update (every 30 seconds)
}

impl PlayAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayAction::Start => "Start",
            PlayAction::Pause => "Pause",
            PlayAction::Resume => "Resume",
            PlayAction::Complete => "Complete",
            PlayAction::Skip => "Skip",
            PlayAction::Stop => "Stop",
            PlayAction::Progress => "Progress",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PlayHistoryResponse {
    pub file_id: i32,
    pub file_title: String,
    pub scholar_name: Option<String>,
    pub played_duration: i32,
    pub total_duration: Option<i32>,
    pub play_position: Option<i32>,
    pub play_action: String,
    pub device_type: Option<String>,
    pub played_at: NaiveDateTime,
}