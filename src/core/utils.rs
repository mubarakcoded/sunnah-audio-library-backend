use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::core::{jwt_auth::JwtClaims, AppConfig};

/// Helper function to extract user ID from optional JWT token
/// Returns Some(user_id) if valid token is provided, None otherwise
pub fn extract_user_id_from_request(req: &HttpRequest, config: &AppConfig) -> Option<i32> {
    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
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
    ).ok()?.claims;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("45:30"), Ok(2730)); // 45*60 + 30
        assert_eq!(parse_duration("1:23:45"), Ok(5025)); // 1*3600 + 23*60 + 45
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("1:2:3:4").is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(2730), "45:30");
        assert_eq!(format_duration(5025), "1:23:45");
        assert_eq!(format_duration(90), "1:30");
    }

    #[test]
    fn test_calculate_total_duration_from_strings() {
        let durations = vec!["45:30".to_string(), "1:23:45".to_string()];
        assert_eq!(calculate_total_duration_from_strings(&durations), Some("2:09:15".to_string()));
        
        let empty_durations: Vec<String> = vec![];
        assert_eq!(calculate_total_duration_from_strings(&empty_durations), None);
    }

    #[test]
    fn test_format_image_url() {
        assert_eq!(format_image_url(Some("test.jpg".to_string())), Some("http://127.0.0.1:8990/api/v1/static/images/test.jpg".to_string()));
        assert_eq!(format_image_url(None), None);
    }

    #[test]
    fn test_format_file_url() {
        assert_eq!(format_file_url("audio.mp3"), "http://127.0.0.1:8990/api/v1/static/uploads/audio.mp3");
    }
}