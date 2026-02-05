use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing::info;

use crate::application::services::HealthService;
use crate::infrastructure::config::AppConfig;

use super::handlers::{health_check, liveness, readiness};

pub struct HttpServerBuilder {
    config: AppConfig,
    health_service: Arc<HealthService>,
}

impl HttpServerBuilder {
    pub fn new(config: AppConfig, health_service: Arc<HealthService>) -> Self {
        Self {
            config,
            health_service,
        }
    }

    pub async fn run(self) -> std::io::Result<()> {
        let addr = self.config.server_addr();
        let health_service = self.health_service;

        info!("Starting HTTP server on {}", addr);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(health_service.clone()))
                .route("/health", web::get().to(health_check))
                .route("/healthz", web::get().to(health_check))
                .route("/livez", web::get().to(liveness))
                .route("/readyz", web::get().to(readiness))
        })
        .bind(&addr)?
        .run()
        .await
    }
}
