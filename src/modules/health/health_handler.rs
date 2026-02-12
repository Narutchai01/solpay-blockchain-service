use axum::Json;

pub struct HealthModel {
    pub status: String,
    pub environment: String,
}

pub trait HealthHandler {
    fn check_health(&self) -> Json<HealthModel>;
}

pub struct HealthHandlerImpl {}

impl HealthHandlerImpl {
    pub fn new() -> Self {
        HealthHandlerImpl {}
    }
}

impl HealthHandler for HealthHandlerImpl {
    fn check_health(&self) -> Json<HealthModel> {
        let health = HealthModel {
            status: "OK".to_string(),
            environment: "production".to_string(),
        };
        Json(health)
    }
}
