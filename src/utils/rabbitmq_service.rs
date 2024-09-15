use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties,
    Result as LapinResult,
};
use serde::Serialize;

pub struct RabbitMQService {
    connection: Connection,
}

impl RabbitMQService {
    pub async fn new(url: &str) -> LapinResult<Self> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        Ok(Self { connection })
    }

    pub async fn publish_transaction<T: Serialize>(
        &self,
        queue_name: &str,
        transaction_data: &T,
    ) -> LapinResult<()> {
        let channel = self.connection.create_channel().await?;

        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let payload = serde_json::to_string(transaction_data)
            .unwrap()
            .into_bytes();

        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}
