use crate::core::AppError;
use crate::models::file_interactions::{
    FileReport, CreateReportRequest, ResolveReportRequest,
    FileLike, LikeFileRequest,
    FileComment, CreateCommentRequest, UpdateCommentRequest, CommentResponse,
    DownloadLog, DownloadStats
};
use sqlx::MySqlPool;
use chrono::{DateTime, Utc};

// File Reports
pub async fn create_file_report(
    pool: &MySqlPool,
    user_id: i32,
    request: &CreateReportRequest,
) -> Result<FileReport, AppError> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_file_reports (user_id, file_id, reason, description, status, created_at)
        VALUES (?, ?, ?, ?, 'pending', ?)
        "#,
        user_id,
        request.file_id,
        request.reason,
        request.description,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let report_id = result.last_insert_id() as i32;
    get_file_report_by_id(pool, report_id).await
}

pub async fn get_file_report_by_id(
    pool: &MySqlPool,
    report_id: i32,
) -> Result<FileReport, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, file_id, reason, description, status, 
               admin_notes, resolved_by, created_at, resolved_at
        FROM tbl_file_reports
        WHERE id = ?
        "#,
        report_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(FileReport {
        id: row.id,
        user_id: row.user_id,
        file_id: row.file_id,
        reason: row.reason,
        description: row.description,
        status: row.status,
        admin_notes: row.admin_notes,
        resolved_by: row.resolved_by,
        created_at: row.created_at.naive_utc(),
        resolved_at: row.resolved_at.map(|dt: DateTime<Utc>| dt.naive_utc()),

    })
}

pub async fn resolve_file_report(
    pool: &MySqlPool,
    report_id: i32,
    admin_user_id: i32,
    request: &ResolveReportRequest,
) -> Result<FileReport, AppError> {
    let now = Utc::now().naive_utc();

    sqlx::query!(
        r#"
        UPDATE tbl_file_reports 
        SET status = ?, admin_notes = ?, resolved_by = ?, resolved_at = ?
        WHERE id = ?
        "#,
        request.status,
        request.admin_notes,
        admin_user_id,
        now,
        report_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_file_report_by_id(pool, report_id).await
}

pub async fn get_pending_reports(
    pool: &MySqlPool,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<FileReport>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, file_id, reason, description, status, 
               admin_notes, resolved_by, created_at, resolved_at
        FROM tbl_file_reports
        WHERE status = 'pending'
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let reports = rows
        .into_iter()
        .map(|row| FileReport {
            id: row.id,
            user_id: row.user_id,
            file_id: row.file_id,
            reason: row.reason,
            description: row.description,
            status: row.status,
            admin_notes: row.admin_notes,
            resolved_by: row.resolved_by,
            created_at: row.created_at.naive_utc(),
            resolved_at: row.resolved_at.map(|dt: DateTime<Utc>| dt.naive_utc()),
        })
        .collect();

    Ok(reports)
}

// File Likes
pub async fn like_file(
    pool: &MySqlPool,
    user_id: i32,
    request: &LikeFileRequest,
) -> Result<FileLike, AppError> {
    let now = Utc::now().naive_utc();

    let _result = sqlx::query!(
        r#"
        INSERT INTO tbl_file_likes (user_id, file_id, created_at)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE created_at = VALUES(created_at)
        "#,
        user_id,
        request.file_id,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_file_like(pool, user_id, request.file_id).await
}

pub async fn unlike_file(
    pool: &MySqlPool,
    user_id: i32,
    file_id: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM tbl_file_likes WHERE user_id = ? AND file_id = ?",
        user_id,
        file_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

pub async fn get_file_like(
    pool: &MySqlPool,
    user_id: i32,
    file_id: i32,
) -> Result<FileLike, AppError> {
    let row = sqlx::query!(
        "SELECT id, user_id, file_id, created_at FROM tbl_file_likes WHERE user_id = ? AND file_id = ?",
        user_id,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(FileLike {
        id: row.id,
        user_id: row.user_id,
        file_id: row.file_id,
        created_at: row.created_at.naive_utc(),
    })
}

pub async fn get_file_likes_count(
    pool: &MySqlPool,
    file_id: i32,
) -> Result<i64, AppError> {
    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM tbl_file_likes WHERE file_id = ?",
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(row.count)
}

pub async fn is_file_liked_by_user(
    pool: &MySqlPool,
    user_id: i32,
    file_id: i32,
) -> Result<bool, AppError> {
    let count = sqlx::query!(
        "SELECT COUNT(*) as count FROM tbl_file_likes WHERE user_id = ? AND file_id = ?",
        user_id,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(count.count > 0)
}

// File Comments
pub async fn create_file_comment(
    pool: &MySqlPool,
    user_id: i32,
    request: &CreateCommentRequest,
) -> Result<FileComment, AppError> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_file_comments (user_id, file_id, parent_id, comment, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        request.file_id,
        request.parent_id,
        request.comment,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let comment_id = result.last_insert_id() as i32;
    get_file_comment_by_id(pool, comment_id).await
}

pub async fn get_file_comment_by_id(
    pool: &MySqlPool,
    comment_id: i32,
) -> Result<FileComment, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, file_id, parent_id, comment, is_approved, created_at, updated_at
        FROM tbl_file_comments
        WHERE id = ?
        "#,
        comment_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(FileComment {
        id: row.id,
        user_id: row.user_id,
        file_id: row.file_id,
        parent_id: row.parent_id,
        comment: row.comment,
        is_approved: row.is_approved.unwrap_or(0) != 0,
        created_at: row.created_at.naive_utc(),
        updated_at: row.updated_at.naive_utc(),
    })
}

pub async fn get_file_comments(
    pool: &MySqlPool,
    file_id: i32,
) -> Result<Vec<CommentResponse>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            c.id, c.parent_id, c.comment, c.is_approved, c.created_at, c.updated_at,
            u.name as user_name
        FROM tbl_file_comments c
        JOIN tbl_users u ON c.user_id = u.id
        WHERE c.file_id = ? AND c.is_approved = 1
        ORDER BY c.created_at ASC
        "#,
        file_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    // Build nested comment structure
    let mut comments_map: std::collections::HashMap<i32, CommentResponse> = std::collections::HashMap::new();
    let mut root_comments = Vec::new();

    for row in rows {
        let comment = CommentResponse {
            id: row.id,
            user_name: row.user_name,
            parent_id: row.parent_id,
            comment: row.comment,
            is_approved: row.is_approved.unwrap_or(0) != 0,
            created_at: row.created_at.naive_utc(),
            updated_at: row.updated_at.naive_utc(),
            replies: Vec::new(),
        };

        if let Some(parent_id) = row.parent_id {
            // This is a reply
            if let Some(parent) = comments_map.get_mut(&parent_id) {
                parent.replies.push(comment);
            }
        } else {
            // This is a root comment
            comments_map.insert(row.id, comment.clone());
            root_comments.push(comment);
        }
    }

    Ok(root_comments)
}

pub async fn update_file_comment(
    pool: &MySqlPool,
    comment_id: i32,
    user_id: i32,
    request: &UpdateCommentRequest,
) -> Result<FileComment, AppError> {
    let now = Utc::now().naive_utc();

    sqlx::query!(
        "UPDATE tbl_file_comments SET comment = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        request.comment,
        now,
        comment_id,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_file_comment_by_id(pool, comment_id).await
}

pub async fn delete_file_comment(
    pool: &MySqlPool,
    comment_id: i32,
    user_id: i32,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM tbl_file_comments WHERE id = ? AND user_id = ?",
        comment_id,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

// Download Logs
pub async fn log_file_download(
    pool: &MySqlPool,
    user_id: i32,
    subscription_id: Option<i32>,
    file_id: i32,
    download_ip: Option<String>,
    user_agent: Option<String>,
) -> Result<DownloadLog, AppError> {
    let now = Utc::now().naive_utc();

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_download_logs (user_id, subscription_id, file_id, download_ip, user_agent, downloaded_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        subscription_id,
        file_id,
        download_ip,
        user_agent,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    // Increment total downloads counter for the file
    sqlx::query!(
        r#"UPDATE tbl_files SET downloads = downloads + 1 WHERE id = ?"#,
        file_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let log_id = result.last_insert_id() as i32;
    get_download_log_by_id(pool, log_id).await
}

pub async fn get_download_log_by_id(
    pool: &MySqlPool,
    log_id: i32,
) -> Result<DownloadLog, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_id, file_id, download_ip, user_agent, downloaded_at
        FROM tbl_download_logs
        WHERE id = ?
        "#,
        log_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(DownloadLog {
        id: row.id,
        user_id: row.user_id.unwrap_or(0),
        subscription_id: row.subscription_id,
        file_id: row.file_id,
        download_ip: row.download_ip,
        user_agent: row.user_agent,
        downloaded_at: row.downloaded_at.naive_utc(),
    })
}

pub async fn get_file_download_stats(
    pool: &MySqlPool,
    file_id: i32,
) -> Result<DownloadStats, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_downloads,
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(CASE WHEN DATE(downloaded_at) = CURDATE() THEN 1 END) as downloads_today,
            COUNT(CASE WHEN YEAR(downloaded_at) = YEAR(CURDATE()) AND MONTH(downloaded_at) = MONTH(CURDATE()) THEN 1 END) as downloads_this_month
        FROM tbl_download_logs
        WHERE file_id = ?
        "#,
        file_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(DownloadStats {
        total_downloads: row.total_downloads,
        unique_users: row.unique_users,
        downloads_today: row.downloads_today,
        downloads_this_month: row.downloads_this_month,
    })
}

pub async fn get_user_download_history(
    pool: &MySqlPool,
    user_id: i32,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<DownloadLog>, AppError> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, subscription_id, file_id, download_ip, user_agent, downloaded_at
        FROM tbl_download_logs
        WHERE user_id = ?
        ORDER BY downloaded_at DESC
        LIMIT ? OFFSET ?
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::db_error)?;

    let logs = rows
        .into_iter()
        .map(|row| DownloadLog {
            id: row.id,
            user_id: row.user_id.unwrap_or(0),
            subscription_id: row.subscription_id,
            file_id: row.file_id,
            download_ip: row.download_ip,
            user_agent: row.user_agent,
            downloaded_at: row.downloaded_at.naive_utc(),
        })
        .collect();

    Ok(logs)
}