use std::sync::Arc;

use axum::{extract::State, Json};

use crate::domain::{HealthCheckProvider, HealthStatus};

pub struct HealthHandler {
    pub health_service: Arc<dyn HealthCheckProvider + Send + Sync>,
}

impl HealthHandler {
    pub fn new(health_service: Arc<dyn HealthCheckProvider + Send + Sync>) -> Self {
        Self { health_service }
    }
}

#[axum::debug_handler]
pub async fn health_check(State(health_handler): State<Arc<HealthHandler>>) -> Json<HealthStatus> {
    let status = health_handler.health_service.get_status().await;
    Json(status)
}
