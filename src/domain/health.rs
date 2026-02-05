use async_trait::async_trait;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct HealthStatus {
    pub is_alive: bool,
    pub mq_connected: bool,
}

#[async_trait]
pub trait HealthCheckProvider: Send + Sync {
    async fn get_status(&self) -> HealthStatus;
    #[allow(dead_code)]
    fn set_mq_status(&self, connected: bool);
}
