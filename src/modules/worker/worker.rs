use futures_util::StreamExt;
use lapin::{BasicProperties, Channel, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;

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

// --- CONSUMER TASK ---
pub async fn run_consumer(channel: Channel) {
    println!("📥 Consumer waiting for messages...");

    let rpc_url = "https://api.devnet.solana.com";

    let client = RpcClient::new(rpc_url.to_string());

    let queue_name = lapin::types::ShortString::from("transactions");

    channel
        .queue_declare(
            queue_name.clone(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("queue_declare");

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "worker_consumer".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("basic_consume");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let msg = String::from_utf8_lossy(&delivery.data);
                println!("📩 Received message: {}", msg);

                // Acknowledge the message
                if let Err(e) = delivery
                    .ack(lapin::options::BasicAckOptions::default())
                    .await
                {
                    eprintln!("❌ Failed to ack message: {:?}", e);
                }
            }

            Err(e) => {
                eprintln!("❌ Error receiving message: {:?}", e);
            }
        }
    }
}
