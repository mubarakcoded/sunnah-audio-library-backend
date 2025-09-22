use crate::core::{AppConfig, RedisHelper, EmailService};
use crate::routes::sunnah_audio_routes;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::MySqlPool;
use std::net::TcpListener;

pub struct SunnahWebServer {
    port: u16,
    server: Server,
}

impl SunnahWebServer {
    pub async fn build(configuration: AppConfig) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.sunnah_audio_server_config.host,
            configuration.sunnah_audio_server_config.port
        );


        let mysql_pool = MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_lazy_with(configuration.mysql.connect());

        let redis = configuration.redis.connect();

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, mysql_pool, redis,  configuration.smtp).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn run(
    listener: TcpListener,
    mysql_pool: MySqlPool,
    redis_client: redis::Client,
    smtp_config: crate::core::config::SmtpConfig,
) -> Result<Server, anyhow::Error> {
    let mysql_pool = Data::new(mysql_pool);
    let redis_client = Data::new(redis_client);
    let redis_helper = Data::new(RedisHelper::new(redis_client.clone()));
    let email_service = Data::new(EmailService::new(smtp_config));
    let _config = crate::core::AppConfig::new().expect("failed to build our appConfig object");

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .configure(sunnah_audio_routes)
            .app_data(mysql_pool.clone())
            .app_data(redis_client.clone())
            .app_data(redis_helper.clone())
            .app_data(email_service.clone())
            .wrap(cors)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
