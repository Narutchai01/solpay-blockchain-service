use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthModel {
    pub status: String,
    pub environment: String,
}

pub struct HealthHandlerImpl {}

impl HealthHandlerImpl {
    pub async fn check_health() -> Json<HealthModel> {
        let health_status = HealthModel {
            status: "OK".to_string(),
            environment: std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
        };
        Json(health_status)
    }
}
