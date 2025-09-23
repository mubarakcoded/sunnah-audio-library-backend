use crate::core::AppError;
use crate::models::users::{User, RegisterRequest, UpdateProfileRequest};
use sqlx::MySqlPool;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::Utc;

pub async fn create_user(
    pool: &MySqlPool,
    request: &RegisterRequest,
) -> Result<User, AppError> {
    let now = Utc::now().naive_utc();
    
    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| AppError::internal_error("Failed to hash password"))?
        .to_string();

    let role = request.role.as_deref().unwrap_or("user");

    let result = sqlx::query!(
        r#"
        INSERT INTO tbl_users (name, email, address, phone, role, password, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
        request.name,
        request.email,
        request.address,
        request.phone,
        role,
        password_hash,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    let user_id = result.last_insert_id() as i32;

    get_user_by_id(pool, user_id).await
}

pub async fn get_user_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, address, phone, role, password, status, 
               created_at as "created_at: chrono::NaiveDateTime", 
               updated_at as "updated_at: chrono::NaiveDateTime"
        FROM tbl_users
        WHERE email = ? AND status = 1
        "#,
        email
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(user)
}

pub async fn get_user_by_id(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, address, phone, role, password, status, 
               created_at as "created_at: chrono::NaiveDateTime", 
               updated_at as "updated_at: chrono::NaiveDateTime"
        FROM tbl_users
        WHERE id = ? AND status = 1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(user)
}

pub async fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| AppError::internal_error("Invalid password"))?;
    
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub async fn update_user_profile(
    pool: &MySqlPool,
    user_id: i32,
    request: &UpdateProfileRequest,
) -> Result<User, AppError> {
    let now = Utc::now().naive_utc();

    // Get current user data
    let current_user = get_user_by_id(pool, user_id).await?;

    let name = request.name.as_deref().unwrap_or(&current_user.name);
    let address = request.address.as_deref().or(current_user.address.as_deref());
    let phone = request.phone.as_deref().or(current_user.phone.as_deref());

    sqlx::query!(
        r#"
        UPDATE tbl_users 
        SET name = ?, address = ?, phone = ?, updated_at = ?
        WHERE id = ?
        "#,
        name,
        address,
        phone,
        now,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    get_user_by_id(pool, user_id).await
}

pub async fn change_user_password(
    pool: &MySqlPool,
    user_id: i32,
    new_password: &str,
) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();
    
    // Hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|_| AppError::internal_error("Failed to hash password"))?
        .to_string();

    sqlx::query!(
        r#"
        UPDATE tbl_users 
        SET password = ?, updated_at = ?
        WHERE id = ?
        "#,
        password_hash,
        now,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}

pub async fn email_exists(
    pool: &MySqlPool,
    email: &str,
) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tbl_users WHERE email = ?",
        email
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(count > 0)
}

pub async fn deactivate_user(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<(), AppError> {
    let now = Utc::now().naive_utc();

    sqlx::query!(
        r#"
        UPDATE tbl_users 
        SET status = 0, updated_at = ?
        WHERE id = ?
        "#,
        now,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::db_error)?;

    Ok(())
}