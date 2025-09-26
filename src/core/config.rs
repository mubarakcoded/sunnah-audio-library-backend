use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use sqlx::mysql::MySqlConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::ConnectOptions;

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub sunnah_audio_server_config: SunnahWebServer,
    pub postgres: PostgresConfig,
    pub mysql: MySqlConfig,
    pub redis: RedisConfig,
    pub jwt_auth_config: JwtAuthConfig,
    pub smtp: SmtpConfig,
    pub app_paths: AppPaths,
}

impl AppConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to find the current dir");
        let config_dir = base_path.join("src/core/configurations");

        let app_environment: Environment = std::env::var("SUNNAH_AUDIO_APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse SUNNAH_AUDIO_APP_ENVIRONMENT");

        // let app_environment: Environment = env::var_os("SUNNAH_AUDIO_APP_ENVIRONMENT")
        // .map(|val| {
        //     val.into_string()
        //         .unwrap_or_else(|_| "local".into())
        //         .try_into()
        //         .expect("Failed to parse SUNNAH_AUDIO_APP_ENVIRONMENT")
        // })
        // .unwrap_or(Environment::Local);

        let configurations = config::Config::builder()
            .add_source(
                config::File::from(config_dir.join(app_environment.as_str())).required(true),
            )
            .build()?;

        configurations.try_deserialize()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SunnahWebServer {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AppPaths {
    pub static_images: String,
    pub static_uploads: String,
    pub static_audio: String,
}

impl AppConfig {
    /// Get the full URL for an image file
    pub fn get_image_url(&self, filename: &str) -> String {
        format!(
            "{}{}/{}",
            self.sunnah_audio_server_config.base_url, self.app_paths.static_images, filename
        )
    }

    /// Get the full URL for an upload file
    pub fn get_upload_url(&self, filename: &str) -> String {
        format!(
            "{}{}/{}",
            self.sunnah_audio_server_config.base_url, self.app_paths.static_uploads, filename
        )
    }

    /// Get the full URL for an audio file
    pub fn get_audio_url(&self, filename: &str) -> String {
        format!(
            "{}{}/{}",
            self.sunnah_audio_server_config.base_url, self.app_paths.static_audio, filename
        )
    }

    /// Get the JWT secret
    pub fn get_jwt_secret(&self) -> &str {
        self.jwt_auth_config.secret.expose_secret()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct RedisConfig {
    pub host: String,
    pub port: String,
    pub password: Option<String>,
}

impl RedisConfig {
    pub fn connect(&self) -> redis::Client {
        let url = format!(
            "redis://:{password}@{host}:{port}",
            password = self.password.as_ref().unwrap_or(&"".to_string()),
            host = self.host,
            port = self.port
        );
        let client = redis::Client::open(url).expect("Failed to connect to Redis");
        client
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostgresConfig {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl PostgresConfig {
    pub fn connect(&self) -> PgConnectOptions {
        let options = PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .database(&self.database_name);

        options.log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MySqlConfig {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl MySqlConfig {
    pub fn connect(&self) -> MySqlConnectOptions {
        let options = MySqlConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .database(&self.database_name);

        options.log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct JwtAuthConfig {
    pub secret: Secret<String>,
    pub token_expiration_time: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub from_email: String,
    pub from_name: String,
}

pub enum Environment {
    Local,
    Sandbox,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Sandbox => "sandbox",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "sandbox" => Ok(Self::Sandbox),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not supported environment. Use either `local`, `sandbox` or `production` ",
                other
            )),
        }
    }
}
