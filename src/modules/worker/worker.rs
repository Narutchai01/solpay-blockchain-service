use lapin::{BasicProperties, Channel, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionData {
    pub tx_id: String,
    pub amount: f64,
}

// --- PRODUCER TASK ---
// จำลองการรับข้อมูลจาก Service A แล้วส่งเข้า RabbitMQ
pub async fn run_producer(channel: Channel) {
    println!("📡 Producer connecting to Service A...");

    // Example data
    let tx = TransactionData {
        tx_id: "tx1234567890".to_string(),
        amount: 100.0,
    };
    let payload = serde_json::to_vec(&tx).unwrap();

    // Declare a queue (idempotent)
    let queue_name = lapin::types::ShortString::from("transactions");
    channel
        .queue_declare(
            queue_name.clone(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("queue_declare");

    // Publish the message
    let confirm = channel
        .basic_publish(
            "".into(),
            queue_name,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await
        .expect("basic_publish")
        .await
        .expect("publisher confirm");

    println!("✅ Sent transaction: {:?}, confirm: {:?}", tx, confirm);
}
