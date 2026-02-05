use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub rabbitmq_url: String,
    pub rabbitmq_queue_name: String,
    pub rabbitmq_exchange_name: String,
    pub rabbitmq_routing_key: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, DomainError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| DomainError::ConfigError(format!("Invalid SERVER_PORT: {}", e)))?,
            rabbitmq_url: std::env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string()),
            rabbitmq_queue_name: std::env::var("RABBITMQ_QUEUE_NAME")
                .unwrap_or_else(|_| "solpay_queue".to_string()),
            rabbitmq_exchange_name: std::env::var("RABBITMQ_EXCHANGE_NAME")
                .unwrap_or_else(|_| "solpay_exchange".to_string()),
            rabbitmq_routing_key: std::env::var("RABBITMQ_ROUTING_KEY")
                .unwrap_or_else(|_| "solpay.#".to_string()),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
