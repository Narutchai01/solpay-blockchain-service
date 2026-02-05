use std::sync::Arc;

use axum::{routing::get, Router};

use crate::handler::{health_check, HealthHandler};
use crate::service::HealthService;

pub fn health_routes() -> Router<()> {
    let health_service = Arc::new(HealthService::new());
    let health_handler = Arc::new(HealthHandler::new(health_service));

    Router::new()
        .route("/health", get(health_check))
        .route("/healthz", get(health_check))
        .with_state(health_handler)
}
