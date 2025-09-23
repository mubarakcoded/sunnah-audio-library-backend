use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub status: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserProfile,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub subscription_status: Option<crate::models::subscriptions::SubscriptionStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpData {
    pub email: String,
    pub otp: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            id: user.id,
            name: user.name,
            email: user.email,
            address: user.address,
            phone: user.phone,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}