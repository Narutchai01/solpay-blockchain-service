use solpay_blockchain_service::core::Config;
use solpay_blockchain_service::modules::mq;
use solpay_blockchain_service::server::{IServer, ServerImpl};

#[tokio::main]
async fn main() {
    let cfg = Config::load();

    let mq_connection = mq::mq::MqClient::connect(&cfg.mq_url);

    mq_connection
        .await
        .expect("Failed to connect to message queue");

    let server = ServerImpl::new(cfg.app_port);

    let server_thread = tokio::spawn(async move { server.start().await });

    println!("Blocking main thread until server is done...");

    let _ = server_thread.await;
}
