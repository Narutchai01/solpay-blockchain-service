use lapin::{Connection, ConnectionProperties, Result};

pub struct MqClient {}

impl MqClient {
    pub async fn connect(uri: &str) -> Result<Connection> {
        let options = ConnectionProperties::default();

        let connection = Connection::connect(uri, options).await?;
        println!("🐰 RabbitMQ Connected successfully!");

        Ok(connection)
    }
}
