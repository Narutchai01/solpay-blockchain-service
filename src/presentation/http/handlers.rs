use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::application::services::HealthService;

pub async fn health_check(health_service: web::Data<Arc<HealthService>>) -> HttpResponse {
    let status = health_service.check_health().await;
    
    if status.rabbitmq_connected {
        HttpResponse::Ok().json(status)
    } else {
        HttpResponse::ServiceUnavailable().json(status)
    }
}

pub async fn liveness(health_service: web::Data<Arc<HealthService>>) -> HttpResponse {
    let status = health_service.liveness();
    HttpResponse::Ok().json(status)
}

pub async fn readiness(health_service: web::Data<Arc<HealthService>>) -> HttpResponse {
    let status = health_service.check_health().await;
    
    if status.rabbitmq_connected {
        HttpResponse::Ok().json(status)
    } else {
        HttpResponse::ServiceUnavailable().json(status)
    }
}
