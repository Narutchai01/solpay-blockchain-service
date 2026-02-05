use crate::domain::{HealthCheckProvider, HealthStatus};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct HealthService {
    mq_connected: AtomicBool,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            mq_connected: AtomicBool::new(true),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for HealthService {
    async fn get_status(&self) -> HealthStatus {
        HealthStatus {
            is_alive: true,
            mq_connected: self.mq_connected.load(Ordering::Relaxed),
        }
    }

    fn set_mq_status(&self, connected: bool) {
        self.mq_connected.store(connected, Ordering::Relaxed);
    }
}
