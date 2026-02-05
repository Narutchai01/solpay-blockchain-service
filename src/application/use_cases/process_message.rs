use async_trait::async_trait;
use tracing::info;

use crate::domain::{
    entities::QueueMessage,
    errors::DomainError,
    ports::MessageHandler,
};

/// Use case for processing messages from the queue
pub struct ProcessMessageUseCase;

impl ProcessMessageUseCase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcessMessageUseCase {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler for ProcessMessageUseCase {
    async fn handle(&self, message: QueueMessage) -> Result<(), DomainError> {
        info!(
            "Processing message: id={}, type={}",
            message.id, message.message_type
        );

        // TODO: Implement actual message processing logic based on message type
        // This is where you would add business logic for handling different message types
        // For example:
        // match message.message_type.as_str() {
        //     "payment.created" => self.handle_payment_created(&message).await,
        //     "payment.confirmed" => self.handle_payment_confirmed(&message).await,
        //     _ => Err(DomainError::ProcessingError(format!("Unknown message type: {}", message.message_type))),
        // }

        info!("Message {} processed successfully", message.id);
        Ok(())
    }
}
