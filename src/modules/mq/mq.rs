use lapin::{Connection, ConnectionProperties};

pub async fn connect(uri: &str) -> lapin::Result<Connection> {
    let conn = Connection::connect(uri, ConnectionProperties::default()).await?;

    println!("🐰 RabbitMQ Connected successfully!");
    Ok(conn)
}
