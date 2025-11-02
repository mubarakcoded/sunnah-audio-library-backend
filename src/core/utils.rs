use crate::core::{jwt_auth::JwtClaims, AppConfig};
use actix_web::{http, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};

use super::{AppError, AppErrorType};
use id3::{Tag, TagLike};
use mp3_metadata;
use std::io::Cursor;

/// Helper function to extract user ID from optional JWT token
/// Returns Some(user_id) if valid token is provided, None otherwise
pub fn extract_user_id_from_request(req: &HttpRequest, config: &AppConfig) -> Option<i32> {
    let token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })?;

    let claims = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(config.get_jwt_secret().as_ref()),
        &Validation::default(),
    )
    .ok()?
    .claims;

    claims.sub.parse().ok()
}

/// Helper function to parse duration string (e.g., "45:30" or "1:23:45")
/// Returns duration in seconds
pub fn parse_duration(duration_str: &str) -> Result<u32, ()> {
    let parts: Vec<&str> = duration_str.split(':').collect();
    match parts.len() {
        2 => {
            // MM:SS format
            let minutes: u32 = parts[0].parse().map_err(|_| ())?;
            let seconds: u32 = parts[1].parse().map_err(|_| ())?;
            Ok(minutes * 60 + seconds)
        }
        3 => {
            // HH:MM:SS format
            let hours: u32 = parts[0].parse().map_err(|_| ())?;
            let minutes: u32 = parts[1].parse().map_err(|_| ())?;
            let seconds: u32 = parts[2].parse().map_err(|_| ())?;
            Ok(hours * 3600 + minutes * 60 + seconds)
        }
        _ => Err(()),
    }
}

/// Helper function to format duration from seconds back to string
/// Returns formatted duration as HH:MM:SS or MM:SS
pub fn format_duration(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

/// Helper function to calculate total duration from a list of duration strings
/// Returns formatted total duration or None if no valid durations found
pub fn calculate_total_duration_from_strings(duration_strings: &[String]) -> Option<String> {
    let mut total_seconds = 0;
    let mut has_valid_duration = false;

    for duration_str in duration_strings {
        if let Ok(duration) = parse_duration(duration_str) {
            total_seconds += duration;
            has_valid_duration = true;
        }
    }

    if has_valid_duration {
        Some(format_duration(total_seconds))
    } else {
        None
    }
}

/// Helper function to format image URL
/// Returns formatted image URL or None if image is None
pub fn format_image_url(image: Option<String>, config: &AppConfig) -> Option<String> {
    image.map(|img| config.get_image_url(&img))
}

/// Helper function to format file URL
/// Returns formatted file URL
pub fn format_file_url(location: &str, config: &AppConfig) -> String {
    config.get_upload_url(location)
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = false;
    for ch in input.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            prev_hyphen = false;
        } else if c.is_ascii() {
            if !prev_hyphen {
                slug.push('-');
                prev_hyphen = true;
            }
        }
    }
    while slug.starts_with('-') {
        slug.remove(0);
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

// Helper function to extract MP3 metadata
pub fn extract_mp3_metadata(file_bytes: &[u8]) -> Result<(String, String), AppError> {
    // Extract duration using mp3-metadata
    let duration_secs = mp3_metadata::read_from_slice(file_bytes)
        .map_err(|e| AppError {
            message: Some("Failed to read MP3 metadata".to_string()),
            cause: Some(e.to_string()),
            error_type: AppErrorType::PayloadValidationError,
        })?
        .duration
        .as_secs();

    let formatted_duration = format_duration(duration_secs.try_into().unwrap());

    // Extract title from ID3 tags
    let cursor = Cursor::new(file_bytes);
    let title = Tag::read_from(cursor)
        .ok()
        .and_then(|tag| tag.title().map(|t| t.to_string()))
        .unwrap_or_else(|| "Untitled".to_string());

    Ok((title, formatted_duration))
}
