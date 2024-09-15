use std::fmt::{Debug, Display};

use sunnah_audio::core::{get_subscriber, init_subscriber, AppConfig};
use sunnah_audio::sunnah_audio_web_server::SunnahWebServer;
use tokio::task::JoinError;

use colored::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let file_appender = tracing_appender::rolling::daily("/var/tmp/log/sunnah_audio", "app");

    let subscriber = get_subscriber("sunnah_audio".into(), "info".into(), file_appender);
    init_subscriber(subscriber);

    let config = AppConfig::new().expect("cant build our appConfig object");

    // let postgres = PgPoolOptions::new()
    //     .acquire_timeout(std::time::Duration::from_secs(5))
    //     .connect_lazy_with(config.postgres.connect());

    // let redis_client = config.redis.connect();

    let sunnah_audio_web_server = SunnahWebServer::build(config.clone())
        .await
        .expect("application could run for some obvious reasons");

    let _x = tokio::spawn(sunnah_audio_web_server.run_until_stopped());

    println!("{}", "-----------------------------------------".green());
    println!(
        "{}",
        format!(
            "🚀 Server started on Addr: {}:{}",
            config.sunnah_audio_server_config.host, config.sunnah_audio_server_config.port
        )
    );
    println!("{}", "-----------------------------------------".green());

    tokio::select! {
        o = _x => {report_exit("xx", o);}
    }
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
