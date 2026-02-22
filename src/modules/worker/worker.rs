use futures_util::StreamExt;
use lapin::{BasicProperties, Channel, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::transaction::VersionedTransaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionData {
    pub tx_id: String,
    pub amount: f64,
    pub base64_tx: Option<String>,
}

// --- PRODUCER TASK ---
// จำลองการรับข้อมูลจาก Service A แล้วส่งเข้า RabbitMQ
pub async fn run_producer(channel: Channel) {
    println!("📡 Producer connecting to Service A...");

    // Example data
    let tx = TransactionData {
        tx_id: "tx1234567890".to_string(),
        amount: 100.0,
        base64_tx: None,
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
// pub async fn run_consumer(channel: Channel) {
//     println!("📥 Consumer waiting for messages...");

//     let rpc_url = "https://api.devnet.solana.com";

//     let client = RpcClient::new(rpc_url.to_string());

//     let queue_name = lapin::types::ShortString::from("transactions");

//     channel
//         .queue_declare(
//             queue_name.clone(),
//             QueueDeclareOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .expect("queue_declare");

//     let mut consumer = channel
//         .basic_consume(
//             queue_name,
//             "worker_consumer".into(),
//             BasicConsumeOptions::default(),
//             FieldTable::default(),
//         )
//         .await
//         .expect("basic_consume");

//     while let Some(delivery) = consumer.next().await {
//         match delivery {
//             Ok(delivery) => match serde_json::from_slice::<TransactionData>(&delivery.data) {
//                 Ok(tx) => {
//                     println!("📩 Received transaction data: {:?}", tx);

//                     match tx.base64_tx {
//                         Some(base64_str) => match base64::decode(&base64_str) {
//                             Ok(decoded) => println!("✅ Decoded base64 transaction: {:?}", decoded),
//                             Err(e) => {
//                                 eprintln!("❌ Failed to decode base64 transaction: {:?}", e);
//                                 continue;
//                             }
//                         },
//                         None => {
//                             eprintln!("❌ No base64 transaction data provided");
//                             continue;
//                         }
//                     };

//                     if let Err(e) = delivery
//                         .ack(lapin::options::BasicAckOptions::default())
//                         .await
//                     {
//                         eprintln!("❌ Failed to ack message: {:?}", e);
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("❌ Failed to deserialize message: {:?}", e);
//                 }
//             },

//             Err(e) => {
//                 eprintln!("❌ Error receiving message: {:?}", e);
//             }
//         }
//     }
// }

pub async fn run_consumer(channel: Channel) {
    println!("📥 Consumer waiting for messages...");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let queue_name = "transactions";

    channel
        .queue_declare(
            queue_name.into(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    let mut consumer = channel
        .basic_consume(
            queue_name.into(),
            "worker_consumer".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to start consumer");

    println!("✅ Consumer started, waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                // 1. Deserialize JSON → TransactionData
                let tx_data = match serde_json::from_slice::<TransactionData>(&delivery.data) {
                    Ok(data) => {
                        println!("📩 Received transaction data: {:?}", data);
                        data
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to deserialize message: {:?}", e);
                        // Nack และ requeue = false เพื่อไม่ให้ loop ซ้ำ
                        let _ = delivery
                            .nack(lapin::options::BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                        continue;
                    }
                };

                // 2. ตรวจสอบ base64_tx field
                let base64_str = match tx_data.base64_tx {
                    Some(s) => s,
                    None => {
                        eprintln!("❌ No base64_tx field in message");
                        let _ = delivery
                            .nack(lapin::options::BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                        continue;
                    }
                };

                // 3. Decode base64 → bytes
                let tx_bytes = match base64::decode(&base64_str) {
                    Ok(bytes) => {
                        println!("✅ Decoded {} bytes", bytes.len());
                        bytes
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to decode base64: {:?}", e);
                        let _ = delivery
                            .nack(lapin::options::BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                        continue;
                    }
                };

                // 4. Deserialize bytes → Solana Transaction
                let transaction: VersionedTransaction = match bincode::deserialize(&tx_bytes) {
                    Ok(tx) => tx,
                    Err(e) => {
                        eprintln!("❌ Failed to deserialize Solana transaction: {:?}", e);
                        let _ = delivery
                            .nack(lapin::options::BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                        continue;
                    }
                };

                // 5. ส่ง Transaction ไปที่ Solana
                match client.send_and_confirm_transaction(&transaction) {
                    Ok(signature) => {
                        println!("🚀 Transaction sent! Signature: {}", signature);
                        println!(
                            "🔗 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                            signature
                        );

                        // Ack เมื่อสำเร็จ
                        if let Err(e) = delivery
                            .ack(lapin::options::BasicAckOptions::default())
                            .await
                        {
                            eprintln!("❌ Failed to ack message: {:?}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to send transaction to Solana: {:?}", e);
                        // Nack + requeue = true เพื่อ retry
                        let _ = delivery
                            .nack(lapin::options::BasicNackOptions {
                                requeue: true,
                                ..Default::default()
                            })
                            .await;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error receiving delivery: {:?}", e);
            }
        }
    }
}
