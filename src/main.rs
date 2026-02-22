use solpay_blockchain_service::core::Config;
use solpay_blockchain_service::modules::{mq, worker};
use solpay_blockchain_service::server::{IServer, ServerImpl};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cfg = Config::load();

    let mq_connection = mq::mq::connect(&cfg.mq_url)
        .await
        .expect("failed to connect to RabbitMQ");

    // let channel_a = mq_connection.create_channel().await.unwrap();
    let channel_b = mq_connection.create_channel().await.unwrap();

    // let prodcuter_thread =
    //     tokio::spawn(async move { worker::worker::run_producer(channel_a).await });

    let consumer_thread =
        tokio::spawn(async move { worker::worker::run_consumer(channel_b).await });

    let server = ServerImpl::new(cfg.app_port);

    let server_thread = tokio::spawn(async move { server.start().await });

    println!("Blocking main thread until server is done...");

    let _ = server_thread.await.unwrap();
    // let _ = prodcuter_thread.await.unwrap();
    let _ = consumer_thread.await.unwrap();
}
