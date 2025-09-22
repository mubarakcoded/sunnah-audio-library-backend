use crate::core::config::SmtpConfig;
use crate::core::AppError;
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;
use std::str::FromStr;

pub struct EmailService {
    smtp_config: SmtpConfig,
}

impl EmailService {
    pub fn new(smtp_config: SmtpConfig) -> Self {
        Self { smtp_config }
    }

    pub async fn send_otp_email(&self, to_email: &str, otp: &str) -> Result<(), AppError> {
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.smtp_config.from_name, self.smtp_config.from_email
        ))
        .map_err(|e| AppError::internal_error(format!("Invalid from email: {}", e)))?;

        let to_mailbox = Mailbox::from_str(to_email)
            .map_err(|e| AppError::internal_error(format!("Invalid to email: {}", e)))?;

        let subject = "Password Reset OTP - Sunnah Audio";
        let body = self.create_otp_email_body(otp);

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::internal_error(format!("Failed to build email: {}", e)))?;

        let credentials = Credentials::new(
            self.smtp_config.username.clone(),
            self.smtp_config.password.expose_secret().clone(),
        );

        let mailer = SmtpTransport::relay(&self.smtp_config.host)
            .map_err(|e| AppError::internal_error(format!("Failed to create SMTP transport: {}", e)))?
            .port(self.smtp_config.port)
            .credentials(credentials)
            .build();

        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!("OTP email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send OTP email to {}: {}", to_email, e);
                Err(AppError::internal_error(format!("Failed to send email: {}", e)))
            }
        }
    }

    fn create_otp_email_body(&self, otp: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Reset OTP</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f4f4f4;
        }}
        .container {{
            background-color: #ffffff;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .logo {{
            font-size: 24px;
            font-weight: bold;
            color: #2c5530;
            margin-bottom: 10px;
        }}
        .otp-container {{
            background-color: #f8f9fa;
            border: 2px solid #2c5530;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            margin: 20px 0;
        }}
        .otp-code {{
            font-size: 32px;
            font-weight: bold;
            color: #2c5530;
            letter-spacing: 5px;
            margin: 10px 0;
        }}
        .warning {{
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 5px;
            padding: 15px;
            margin: 20px 0;
        }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            font-size: 12px;
            color: #666;
        }}
        .button {{
            display: inline-block;
            padding: 12px 24px;
            background-color: #2c5530;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            margin: 10px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🎧 Sunnah Audio</div>
            <h2>Password Reset Request</h2>
        </div>
        
        <p>Hello,</p>
        
        <p>We received a request to reset your password for your Sunnah Audio account. Use the OTP (One-Time Password) below to reset your password:</p>
        
        <div class="otp-container">
            <p><strong>Your OTP Code:</strong></p>
            <div class="otp-code">{}</div>
            <p><small>This code is valid for 10 minutes only</small></p>
        </div>
        
        <div class="warning">
            <strong>⚠️ Security Notice:</strong>
            <ul style="margin: 10px 0; padding-left: 20px;">
                <li>This OTP will expire in <strong>10 minutes</strong></li>
                <li>Do not share this code with anyone</li>
                <li>If you didn't request this reset, please ignore this email</li>
                <li>For security, this code can only be used once</li>
            </ul>
        </div>
        
        <p>To reset your password:</p>
        <ol>
            <li>Go to the password reset page</li>
            <li>Enter your email address</li>
            <li>Enter the OTP code above</li>
            <li>Create your new password</li>
        </ol>
        
        <p>If you have any questions or need assistance, please contact our support team.</p>
        
        <p>Best regards,<br>
        The Sunnah Audio Team</p>
        
        <div class="footer">
            <p>This is an automated message. Please do not reply to this email.</p>
            <p>© 2024 Sunnah Audio. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            otp
        )
    }

    pub async fn send_password_reset_confirmation(&self, to_email: &str) -> Result<(), AppError> {
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.smtp_config.from_name, self.smtp_config.from_email
        ))
        .map_err(|e| AppError::internal_error(format!("Invalid from email: {}", e)))?;

        let to_mailbox = Mailbox::from_str(to_email)
            .map_err(|e| AppError::internal_error(format!("Invalid to email: {}", e)))?;

        let subject = "Password Reset Successful - Sunnah Audio";
        let body = self.create_password_reset_confirmation_body();

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::internal_error(format!("Failed to build email: {}", e)))?;

        let credentials = Credentials::new(
            self.smtp_config.username.clone(),
            self.smtp_config.password.expose_secret().clone(),
        );

        let mailer = SmtpTransport::relay(&self.smtp_config.host)
            .map_err(|e| AppError::internal_error(format!("Failed to create SMTP transport: {}", e)))?
            .port(self.smtp_config.port)
            .credentials(credentials)
            .build();

        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!("Password reset confirmation email sent to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send confirmation email to {}: {}", to_email, e);
                Err(AppError::internal_error(format!("Failed to send email: {}", e)))
            }
        }
    }

    fn create_password_reset_confirmation_body(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Reset Successful</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f4f4f4;
        }}
        .container {{
            background-color: #ffffff;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .logo {{
            font-size: 24px;
            font-weight: bold;
            color: #2c5530;
            margin-bottom: 10px;
        }}
        .success {{
            background-color: #d4edda;
            border: 1px solid #c3e6cb;
            border-radius: 5px;
            padding: 15px;
            margin: 20px 0;
            text-align: center;
        }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            font-size: 12px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🎧 Sunnah Audio</div>
            <h2>Password Reset Successful</h2>
        </div>
        
        <div class="success">
            <h3>✅ Your password has been successfully reset!</h3>
        </div>
        
        <p>Hello,</p>
        
        <p>This email confirms that your Sunnah Audio account password has been successfully reset.</p>
        
        <p><strong>What happens next:</strong></p>
        <ul>
            <li>You can now log in with your new password</li>
            <li>All active sessions have been terminated for security</li>
            <li>You may need to log in again on all your devices</li>
        </ul>
        
        <p><strong>Security reminder:</strong></p>
        <ul>
            <li>Keep your password secure and don't share it with anyone</li>
            <li>Use a strong, unique password for your account</li>
            <li>If you didn't make this change, contact support immediately</li>
        </ul>
        
        <p>If you have any questions or concerns, please don't hesitate to contact our support team.</p>
        
        <p>Best regards,<br>
        The Sunnah Audio Team</p>
        
        <div class="footer">
            <p>This is an automated message. Please do not reply to this email.</p>
            <p>© 2024 Sunnah Audio. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#
        )
    }
}