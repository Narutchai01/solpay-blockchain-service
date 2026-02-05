use async_trait::async_trait;
use futures_lite::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::domain::{
    entities::QueueMessage,
    errors::DomainError,
    ports::{MessageConsumer, MessageHandler},
};
use crate::infrastructure::config::AppConfig;

pub struct RabbitMQConsumer {
    config: AppConfig,
    connection: Arc<RwLock<Option<Connection>>>,
    channel: Arc<RwLock<Option<Channel>>>,
    handler: Arc<dyn MessageHandler>,
    is_running: Arc<RwLock<bool>>,
}

impl RabbitMQConsumer {
    pub fn new(config: AppConfig, handler: Arc<dyn MessageHandler>) -> Self {
        Self {
            config,
            connection: Arc::new(RwLock::new(None)),
            channel: Arc::new(RwLock::new(None)),
            handler,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    async fn connect(&self) -> Result<(), DomainError> {
        info!("Connecting to RabbitMQ at {}", self.config.rabbitmq_url);

        let connection =
            Connection::connect(&self.config.rabbitmq_url, ConnectionProperties::default()).await?;

        let channel = connection.create_channel().await?;

        // Declare the queue
        channel
            .queue_declare(
                &self.config.rabbitmq_queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Declare and bind exchange if specified
        if !self.config.rabbitmq_exchange_name.is_empty() {
            // Declare the exchange first
            channel
                .exchange_declare(
                    &self.config.rabbitmq_exchange_name,
                    lapin::ExchangeKind::Topic,
                    lapin::options::ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;

            // Then bind queue to exchange
            channel
                .queue_bind(
                    &self.config.rabbitmq_queue_name,
                    &self.config.rabbitmq_exchange_name,
                    &self.config.rabbitmq_routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        *self.connection.write().await = Some(connection);
        *self.channel.write().await = Some(channel);

        info!("Successfully connected to RabbitMQ");
        Ok(())
    }
}

#[async_trait]
impl MessageConsumer for RabbitMQConsumer {
    async fn start_consuming(&self) -> Result<(), DomainError> {
        self.connect().await?;

        *self.is_running.write().await = true;

        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| DomainError::ConnectionError("Channel not initialized".to_string()))?;

        let mut consumer = channel
            .basic_consume(
                &self.config.rabbitmq_queue_name,
                "solpay-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        drop(channel_guard);

        info!(
            "Started consuming from queue: {}",
            self.config.rabbitmq_queue_name
        );

        while let Some(delivery_result) = consumer.next().await {
            if !*self.is_running.read().await {
                break;
            }

            match delivery_result {
                Ok(delivery) => {
                    let payload = String::from_utf8_lossy(&delivery.data).to_string();

                    match serde_json::from_str::<QueueMessage>(&payload) {
                        Ok(message) => {
                            info!("Received message: {:?}", message.id);

                            match self.handler.handle(message).await {
                                Ok(_) => {
                                    if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                        error!("Failed to ack message: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to handle message: {}", e);
                                    if let Err(e) = delivery
                                        .nack(BasicNackOptions {
                                            requeue: true,
                                            ..Default::default()
                                        })
                                        .await
                                    {
                                        error!("Failed to nack message: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {}, payload: {}", e, payload);
                            // Ack invalid messages to prevent blocking
                            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                error!("Failed to ack invalid message: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                }
            }
        }

        info!("Consumer stopped");
        Ok(())
    }

    async fn stop_consuming(&self) -> Result<(), DomainError> {
        *self.is_running.write().await = false;

        if let Some(channel) = self.channel.write().await.take() {
            channel.close(200, "Normal shutdown").await?;
        }

        if let Some(connection) = self.connection.write().await.take() {
            connection.close(200, "Normal shutdown").await?;
        }

        info!("Consumer stopped gracefully");
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        if let Some(connection) = self.connection.read().await.as_ref() {
            connection.status().connected()
        } else {
            false
        }
    }
}
