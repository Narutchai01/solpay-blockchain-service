use axum::Router;

pub trait IHealthRoute {
    fn register(&self) -> Router;
}

#[derive(Clone)]
pub struct HealthRouteImpl {
    pub path: String,
    pub router: Router,
}

impl HealthRouteImpl {
    #[allow(dead_code)]
    pub fn new(path: String, router: Router) -> Self {
        HealthRouteImpl { path, router }
    }
}

impl IHealthRoute for HealthRouteImpl {
    fn register(&self) -> Router {
        let router = self.router.clone();
        router.route(&self.path, axum::routing::get(|| async { "OK" }))
    }
}
