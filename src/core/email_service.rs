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

    fn create_smtp_transport(&self) -> Result<SmtpTransport, AppError> {
        let credentials = Credentials::new(
            self.smtp_config.username.clone(),
            self.smtp_config.password.expose_secret().clone(),
        );

        // For Mailtrap (port 2525), use STARTTLS instead of direct TLS
        let mailer = if self.smtp_config.port == 2525 {
            // Mailtrap configuration - use STARTTLS
            SmtpTransport::starttls_relay(&self.smtp_config.host)
                .map_err(|e| {
                    AppError::internal_error(format!("Failed to create SMTP transport: {}", e))
                })?
                .port(self.smtp_config.port)
                .credentials(credentials)
                .build()
        } else {
            // Standard SMTP configuration
            SmtpTransport::relay(&self.smtp_config.host)
                .map_err(|e| {
                    AppError::internal_error(format!("Failed to create SMTP transport: {}", e))
                })?
                .port(self.smtp_config.port)
                .credentials(credentials)
                .build()
        };

        Ok(mailer)
    }

    pub async fn send_otp_email(&self, to_email: &str, otp: &str) -> Result<(), AppError> {
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.smtp_config.from_name, self.smtp_config.from_email
        ))
        .map_err(|e| AppError::internal_error(format!("Invalid from email: {}", e)))?;

        let to_mailbox = Mailbox::from_str(to_email)
            .map_err(|e| AppError::internal_error(format!("Invalid to email: {}", e)))?;

        let subject = "Password Reset OTP - Muryar Sunnah";
        let body = self.create_otp_email_body(otp);

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::internal_error(format!("Failed to build email: {}", e)))?;

        let mailer = self.create_smtp_transport()?;

        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!("OTP email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send OTP email to {}: {}", to_email, e);
                Err(AppError::internal_error(format!(
                    "Failed to send email: {}",
                    e
                )))
            }
        }
    }

    pub async fn send_password_reset_confirmation(&self, to_email: &str) -> Result<(), AppError> {
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.smtp_config.from_name, self.smtp_config.from_email
        ))
        .map_err(|e| AppError::internal_error(format!("Invalid from email: {}", e)))?;

        let to_mailbox = Mailbox::from_str(to_email)
            .map_err(|e| AppError::internal_error(format!("Invalid to email: {}", e)))?;

        let subject = "Password Reset Successful - Muryar Sunnah";
        let body = self.create_confirmation_email_body();

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::internal_error(format!("Failed to build email: {}", e)))?;

        let mailer = self.create_smtp_transport()?;

        match mailer.send(&email) {
            Ok(_) => {
                tracing::info!(
                    "Password reset confirmation email sent successfully to: {}",
                    to_email
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send confirmation email to {}: {}", to_email, e);
                Err(AppError::internal_error(format!(
                    "Failed to send email: {}",
                    e
                )))
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
            background-color: white;
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
            border: 2px dashed #2c5530;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            margin: 20px 0;
        }}
        .otp-code {{
            font-size: 32px;
            font-weight: bold;
            color: #2c5530;
            letter-spacing: 8px;
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
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            font-size: 12px;
            color: #666;
            text-align: center;
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
            <div class="logo">🎧 Muryar Sunnah</div>
            <h1>Password Reset Request</h1>
        </div>
        
        <p>Assalamu Alaikum,</p>
        
        <p>We received a request to reset your password for your Muryar Sunnah account. Use the OTP code below to complete your password reset:</p>
        
        <div class="otp-container">
            <p><strong>Your OTP Code:</strong></p>
            <div class="otp-code">{}</div>
            <p><small>This code will expire in 10 minutes</small></p>
        </div>
        
        <div class="warning">
            <strong>⚠️ Security Notice:</strong>
            <ul>
                <li>Never share this OTP code with anyone</li>
                <li>Our team will never ask for your OTP via phone or email</li>
                <li>If you didn't request this reset, please ignore this email</li>
                <li>This code expires in 10 minutes for your security</li>
            </ul>
        </div>
        
        <p><strong>How to use this OTP:</strong></p>
        <ol>
            <li>Go back to the password reset page</li>
            <li>Enter this OTP code: <strong>{}</strong></li>
            <li>Create your new password</li>
            <li>Click "Reset Password" to complete the process</li>
        </ol>
        
        <div class="footer">
            <p>This is an automated message from Muryar Sunnah. Please do not reply to this email.</p>
            <p>If you have any questions, please contact our support team.</p>
        </div>
    </div>
</body>
</html>
"#,
            otp, otp
        )
    }

    fn create_confirmation_email_body(&self) -> String {
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Reset Successful</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f4f4f4;
        }
        .container {
            background-color: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        .logo {
            font-size: 24px;
            font-weight: bold;
            color: #2c5530;
            margin-bottom: 10px;
        }
        .success-icon {
            font-size: 48px;
            color: #28a745;
            margin: 20px 0;
        }
        .footer {
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            font-size: 12px;
            color: #666;
            text-align: center;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🎧 Muryar Sunnah</div>
            <div class="success-icon">✅</div>
            <h1>Password Reset Successful</h1>
        </div>
        
        <p>Assalamu Alaikum,</p>
        
        <p>Your password has been successfully reset for your Muryar Sunnah account.</p>
        
        <p><strong>What happens next:</strong></p>
        <ul>
            <li>You can now log in with your new password</li>
            <li>All your account data and preferences remain unchanged</li>
            <li>Your active sessions on other devices have been logged out for security</li>
        </ul>
        
        <p><strong>Security Reminders:</strong></p>
        <ul>
            <li>Keep your password secure and don't share it with anyone</li>
            <li>Use a strong, unique password for your account</li>
            <li>If you notice any suspicious activity, contact us immediately</li>
        </ul>
        
        <p>If you didn't make this change, please contact our support team immediately.</p>
        
        <div class="footer">
            <p>This is an automated message from Muryar Sunnah. Please do not reply to this email.</p>
            <p>If you have any questions, please contact our support team.</p>
        </div>
    </div>
</body>
</html>
"#.to_string()
    }
}
