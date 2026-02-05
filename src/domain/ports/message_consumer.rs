use async_trait::async_trait;

use crate::domain::errors::DomainError;

/// Port for message queue consumer (driven port)
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Start consuming messages from the queue
    async fn start_consuming(&self) -> Result<(), DomainError>;

    /// Stop consuming messages
    async fn stop_consuming(&self) -> Result<(), DomainError>;

    /// Check if the consumer is healthy/connected
    async fn is_healthy(&self) -> bool;
}
