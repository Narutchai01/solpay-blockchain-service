use futures_util::stream::StreamExt;
use lapin::{BasicProperties, Channel, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionData {
    pub tx_id: String,
    pub amount: f64,
}

// --- PRODUCER TASK ---
// จำลองการรับข้อมูลจาก Service A แล้วส่งเข้า RabbitMQ
pub async fn run_producer(channel: Channel) {
    println!("📡 Producer connecting to Service A...");

    // Loop นี้สมมติว่าเป็น Listener ที่รับข้อมูลจาก Service A ตลอดเวลา
    loop {
        // 1. รับข้อมูลจาก Service A (สมมติว่ารับมาได้)
        // let data = service_a_client.receive().await;
        let data = TransactionData {
            tx_id: "tx_123".to_string(),
            amount: 1.5,
        };

        let payload = serde_json::to_vec(&data).unwrap();

        // 2. ส่งเข้า RabbitMQ
        channel
            .basic_publish(
                "",
                "solana_queue",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .unwrap();

        println!("📤 Sent to Queue: {:?}", data);

        // จำลอง Delay (ในงานจริงอาจจะรอข้อมูลจาก Stream)
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// --- CONSUMER TASK ---
// ดึงจาก RabbitMQ แล้วยิงไป Service B
pub async fn run_consumer(channel: Channel) {
    println!("⚙️ Consumer ready to call Service B...");

    let mut consumer = channel
        .basic_consume(
            "solana_queue",
            "worker_1",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let http_client = reqwest::Client::new();

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let data: TransactionData = serde_json::from_slice(&delivery.data).unwrap();

            println!("📥 Received from Queue: {:?}", data);

            // 3. ยิงไป Service B (เช่น Service ภายนอก)
            let res = http_client
                .post("https://service-b.internal/process")
                .json(&data)
                .send()
                .await;

            match res {
                Ok(_) => {
                    println!("✅ Sent to Service B Success");
                    delivery.ack(BasicAckOptions::default()).await.unwrap();
                }
                Err(e) => {
                    println!("❌ Failed to call Service B: {}", e);
                    // ตัดสินใจว่าจะ Nack (เพื่อให้คิวส่งมาใหม่) หรือทิ้งไป
                    // delivery.nack(BasicNackOptions::default()).await.unwrap();
                }
            }
        }
    }
}
