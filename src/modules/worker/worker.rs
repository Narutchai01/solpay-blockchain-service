use futures_util::StreamExt;
use lapin::{BasicProperties, Channel, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::transaction::VersionedTransaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionData {
    pub tx_id: String,
    pub base64_tx: Option<String>,
    pub metadata: Option<MetaData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    pub transaction_type: String,
    pub amount_thb: f64,
    pub amount_usdc: f64,
    pub account_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerMessage {
    pub tx_id: String,
    pub status: String,
}

// --- PRODUCER TASK ---
// ส่งผลลัพธ์กลับเข้า RabbitMQ หลังจาก submit เข้า Solana สำเร็จ
pub async fn run_producer(channel: &Channel, msg: WorkerMessage) {
    println!("📡 Publishing result to queue...");

    let payload = serde_json::to_vec(&msg).unwrap();

    // Declare a result queue (idempotent)
    let queue_name = "core.transaction.status.update";
    channel
        .queue_declare(
            queue_name.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("queue_declare");

    // Publish the message
    let confirm = channel
        .basic_publish(
            "".into(),
            queue_name.into(),
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await
        .expect("basic_publish")
        .await
        .expect("publisher confirm");

    println!("✅ Published result: {:?}, confirm: {:?}", msg, confirm);
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

    let queue_name = "solana-worker.tx.submit";

    let options = QueueDeclareOptions {
        durable: true, // เปลี่ยนตรงนี้ให้เป็น true
        ..Default::default()
    };

    channel
        .queue_declare(queue_name.into(), options, FieldTable::default())
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

                        let msg = WorkerMessage {
                            tx_id: tx_data.tx_id.clone(),
                            status: "BLOCKCHAIN_COMPLETED".to_string(),
                        };

                        run_producer(&channel, msg).await;

                        // Ack เมื่อสำเร็จ
                        if let Err(e) = delivery
                            .ack(lapin::options::BasicAckOptions::default())
                            .await
                        {
                            eprintln!("❌ Failed to ack message: {:?}", e);
                            let _ = delivery
                                .nack(lapin::options::BasicNackOptions {
                                    requeue: false,
                                    ..Default::default()
                                })
                                .await;
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to send transaction to Solana: {:?}", e);

                        let msg = WorkerMessage {
                            tx_id: tx_data.tx_id.clone(),
                            status: "BLOCKCHAIN_FAILED".to_string(),
                        };

                        run_producer(&channel, msg).await;

                        // Ack เพื่อ clear จาก queue (ไม่ retry เพราะส่งสถานะกลับแล้ว)
                        let _ = delivery
                            .ack(lapin::options::BasicAckOptions::default())
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
