use serde::Serialize;
use std::sync::Arc;

use crate::domain::ports::MessageConsumer;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub version: String,
    pub rabbitmq_connected: bool,
}

pub struct HealthService {
    consumer: Arc<dyn MessageConsumer>,
}

impl HealthService {
    pub fn new(consumer: Arc<dyn MessageConsumer>) -> Self {
        Self { consumer }
    }

    pub async fn check_health(&self) -> HealthStatus {
        let rabbitmq_connected = self.consumer.is_healthy().await;

        HealthStatus {
            status: if rabbitmq_connected { "healthy".to_string() } else { "degraded".to_string() },
            service: "solpay-blockchain-service".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            rabbitmq_connected,
        }
    }

    pub fn liveness(&self) -> HealthStatus {
        HealthStatus {
            status: "alive".to_string(),
            service: "solpay-blockchain-service".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            rabbitmq_connected: false, // Not checked for liveness
        }
    }
}
