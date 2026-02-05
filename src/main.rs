use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use solpay_blockchain_service::{
    application::{services::HealthService, use_cases::ProcessMessageUseCase},
    domain::ports::MessageConsumer,
    infrastructure::{config::AppConfig, rabbitmq::RabbitMQConsumer},
    presentation::http::HttpServerBuilder,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "solpay_blockchain_service=debug,actix_web=info,lapin=info".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Solpay Blockchain Service");

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Create message handler (use case)
    let message_handler = Arc::new(ProcessMessageUseCase::new());

    // Create RabbitMQ consumer
    let consumer: Arc<dyn MessageConsumer> =
        Arc::new(RabbitMQConsumer::new(config.clone(), message_handler));

    // Create health service
    let health_service = Arc::new(HealthService::new(consumer.clone()));

    // Clone consumer for the background task
    let consumer_clone = consumer.clone();

    // Spawn RabbitMQ consumer in a background task
    let consumer_handle = tokio::spawn(async move {
        loop {
            info!("Starting RabbitMQ consumer...");
            if let Err(e) = consumer_clone.start_consuming().await {
                error!("Consumer error: {}. Reconnecting in 5 seconds...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    // Create and run HTTP server
    let server = HttpServerBuilder::new(config.clone(), health_service);

    info!("Service is ready. HTTP server running on {}", config.server_addr());

    // Run HTTP server (this blocks until shutdown)
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("HTTP server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
    }

    // Cleanup
    consumer_handle.abort();
    info!("Service stopped");

    Ok(())
}

