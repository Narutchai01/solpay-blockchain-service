use async_trait::async_trait;

use crate::domain::entities::QueueMessage;
use crate::domain::errors::DomainError;

/// Port for handling messages (use case interface)
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, message: QueueMessage) -> Result<(), DomainError>;
}
