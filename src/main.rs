use solpay_blockchain_service::server::{IServer, ServerImpl};

#[tokio::main]
async fn main() {
    let server = ServerImpl::new(8080);

    let server_thread = tokio::spawn(async move { server.start().await });

    println!("Blocking main thread until server is done...");

    let _ = server_thread.await;
}
