use crate::core::AppError;
use crate::models::playlists::{
    AddToPlaylistRequest, CreatePlaylistRequest, Playlist, PlaylistFile, PlaylistFileResponse,
    PlaylistResponse, UpdatePlaylistRequest,
};
use chrono::Utc;
use sqlx::MySqlPool;

// Create playlist
pub async fn create_playlist(
    pool: &MySqlPool,
    user_id: i32,
    request: &CreatePlaylistRequest,
) -> Result<Playlist, AppError> {
    let now = Utc::now().naive_utc();
    let is_public = request.is_public.unwrap_or(false);

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_playlists (user_id, name, description, is_public, cover_image, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        request.name,
        request.description,
        is_public,
        request.cover_image,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let playlist_id = result.last_insert_id() as i32;
    get_playlist_by_id(pool, playlist_id).await
}

// Get playlist by ID
pub async fn get_playlist_by_id(pool: &MySqlPool, playlist_id: i32) -> Result<Playlist, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, name, description, is_public, cover_image, 
               total_files, total_duration, created_at, updated_at
        FROM tbl_playlists
        WHERE id = ?
        "#,
        playlist_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(Playlist {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        description: row.description,
        is_public: row.is_public.unwrap_or(0) != 0,
        cover_image: row.cover_image,
        total_files: row.total_files.unwrap_or(0),
        total_duration: row.total_duration.unwrap_or(0),
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    })
}

// Get user playlists
pub async fn get_user_playlists(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<PlaylistResponse>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.name, p.description, p.is_public, p.cover_image,
               p.total_files, p.total_duration, p.created_at, p.updated_at,
               u.name as owner_name
        FROM tbl_playlists p
        JOIN tbl_users u ON p.user_id = u.id
        WHERE p.user_id = ?
        ORDER BY p.updated_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let playlists = rows
        .into_iter()
        .map(|row| PlaylistResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            is_public: row.is_public.unwrap_or(0) != 0,
            cover_image: row.cover_image,
            total_files: row.total_files.unwrap_or(0),
            total_duration: row.total_duration.unwrap_or(0),
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
            owner_name: row.owner_name,
        })
        .collect();

    Ok(playlists)
}

// Get public playlists
pub async fn get_public_playlists(
    pool: &MySqlPool,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<PlaylistResponse>, AppError> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.name, p.description, p.is_public, p.cover_image,
               p.total_files, p.total_duration, p.created_at, p.updated_at,
               u.name as owner_name
        FROM tbl_playlists p
        JOIN tbl_users u ON p.user_id = u.id
        WHERE p.is_public = 1 AND p.total_files > 0
        ORDER BY p.updated_at DESC
        LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let playlists = rows
        .into_iter()
        .map(|row| PlaylistResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            is_public: row.is_public.unwrap_or(0) != 0,
            cover_image: row.cover_image,
            total_files: row.total_files.unwrap_or(0),
            total_duration: row.total_duration.unwrap_or(0),
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
            owner_name: row.owner_name,
        })
        .collect();

    Ok(playlists)
}

// Update playlist
pub async fn update_playlist(
    pool: &MySqlPool,
    playlist_id: i32,
    user_id: i32,
    request: &UpdatePlaylistRequest,
) -> Result<Playlist, AppError> {
    let now = Utc::now().naive_utc();

    // Check if there are any fields to update
    if request.name.is_none()
        && request.description.is_none()
        && request.is_public.is_none()
        && request.cover_image.is_none()
    {
        return get_playlist_by_id(pool, playlist_id).await;
    }

    // Use individual update queries for each field
    if let Some(name) = &request.name {
        sqlx::query!(
            "UPDATE tbl_playlists SET name = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            name,
            now,
            playlist_id,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }
    if let Some(description) = &request.description {
        sqlx::query!(
            "UPDATE tbl_playlists SET description = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            description,
            now,
            playlist_id,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }
    if let Some(is_public) = request.is_public {
        sqlx::query!(
            "UPDATE tbl_playlists SET is_public = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            is_public,
            now,
            playlist_id,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }
    if let Some(cover_image) = &request.cover_image {
        sqlx::query!(
            "UPDATE tbl_playlists SET cover_image = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            cover_image,
            now,
            playlist_id,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::db_error)?;
    }

    get_playlist_by_id(pool, playlist_id).await
}

// Delete playlist
pub async fn delete_playlist(
    pool: &MySqlPool,
    playlist_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM tbl_playlists WHERE id = ? AND user_id = ?",
        playlist_id,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

// Add file to playlist
pub async fn add_file_to_playlist(
    pool: &MySqlPool,
    playlist_id: i32,
    user_id: i32,
    request: &AddToPlaylistRequest,
) -> Result<PlaylistFile, AppError> {
    // Verify playlist ownership
    let playlist = get_playlist_by_id(pool, playlist_id).await?;
    if playlist.user_id != user_id {
        return Err(AppError::forbidden_error("You don't own this playlist"));
    }

    let now = Utc::now().naive_utc();
    let sort_order = request.sort_order.unwrap_or_else(|| {
        // Get next sort order
        0 // This should be calculated from existing files
    });

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_playlist_files (playlist_id, file_id, sort_order, created_at)
        VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE sort_order = VALUES(sort_order)
        "#,
        playlist_id,
        request.file_id,
        sort_order,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    // Update playlist stats
    update_playlist_stats(pool, playlist_id).await?;

    let playlist_file_id = result.last_insert_id() as i32;
    get_playlist_file_by_id(pool, playlist_file_id).await
}

// Remove file from playlist
pub async fn remove_file_from_playlist(
    pool: &MySqlPool,
    playlist_id: i32,
    file_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    // Verify playlist ownership
    let playlist = get_playlist_by_id(pool, playlist_id).await?;
    if playlist.user_id != user_id {
        return Err(AppError::forbidden_error("You don't own this playlist"));
    }

    sqlx::query!(
        "DELETE FROM tbl_playlist_files WHERE playlist_id = ? AND file_id = ?",
        playlist_id,
        file_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    // Update playlist stats
    update_playlist_stats(pool, playlist_id).await?;

    Ok(())
}

// Get playlist files
pub async fn get_playlist_files(
    pool: &MySqlPool,
    config: &crate::core::AppConfig,
    playlist_id: i32,
) -> Result<Vec<PlaylistFileResponse>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            pf.file_id, f.name as file_title, f.location, s.name as scholar_name,
            s.image as scholar_image, b.image as book_image,
            f.duration, pf.sort_order, pf.created_at as added_at
        FROM tbl_playlist_files pf
        JOIN tbl_files f ON pf.file_id = f.id
        LEFT JOIN tbl_scholars s ON f.scholar = s.id
        LEFT JOIN tbl_books b ON f.book = b.id
        WHERE pf.playlist_id = ?
        ORDER BY pf.sort_order ASC, pf.created_at ASC
        "#,
        playlist_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let files = rows
        .into_iter()
        .map(|row| PlaylistFileResponse {
            file_id: row.file_id,
            file_title: row.file_title,
            file_url: config.get_upload_url(&row.location),
            scholar_name: row.scholar_name.clone(),
            scholar_image: row.scholar_image.map(|img| config.get_image_url(&img)),
            book_image: row.book_image.map(|img| config.get_image_url(&img)),
            duration: row.duration,
            sort_order: row.sort_order.unwrap_or(0),
            added_at: row.added_at.naive_utc(),
        }) 
        .collect();

    Ok(files)
}

// Helper functions
async fn get_playlist_file_by_id(
    pool: &MySqlPool,
    playlist_file_id: i32,
) -> Result<PlaylistFile, AppError> {
    let row = sqlx::query!(
        "SELECT id, playlist_id, file_id, sort_order, created_at FROM tbl_playlist_files WHERE id = ?",
        playlist_file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(PlaylistFile {
        id: row.id,
        playlist_id: row.playlist_id,
        file_id: row.file_id,
        sort_order: row.sort_order.unwrap_or(0),
        created_at: row.created_at.naive_utc(),
    })
}

async fn update_playlist_stats(pool: &MySqlPool, playlist_id: i32) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();

    // Get the count of files
    let total_files: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_playlist_files WHERE playlist_id = ?",
        playlist_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    // Get all file durations to calculate total
    let durations = sqlx::query_scalar!(
        r#"
        SELECT f.duration 
        FROM tbl_playlist_files pf 
        JOIN tbl_files f ON pf.file_id = f.id 
        WHERE pf.playlist_id = ?
        "#,
        playlist_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Calculate total duration in seconds from duration strings
    let total_duration_seconds = {
        let mut total_seconds: u32 = 0;
        for duration_str in &durations {
            // Parse duration string (e.g., "2:53" or "1:23:45")
            let parts: Vec<&str> = duration_str.split(':').collect();
            let seconds = match parts.len() {
                2 => {
                    // MM:SS format
                    let minutes: u32 = parts[0].parse().unwrap_or(0);
                    let secs: u32 = parts[1].parse().unwrap_or(0);
                    minutes * 60 + secs
                }
                3 => {
                    // HH:MM:SS format
                    let hours: u32 = parts[0].parse().unwrap_or(0);
                    let minutes: u32 = parts[1].parse().unwrap_or(0);
                    let secs: u32 = parts[2].parse().unwrap_or(0);
                    hours * 3600 + minutes * 60 + secs
                }
                _ => 0,
            };
            total_seconds += seconds;
        }
        total_seconds as i32
    };

    // Update playlist stats
    sqlx::query!(
        r#"
        UPDATE tbl_playlists 
        SET 
            total_files = ?,
            total_duration = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        total_files,
        total_duration_seconds,
        now,
        playlist_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}
