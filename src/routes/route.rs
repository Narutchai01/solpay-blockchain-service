use crate::routes::health_route::{HealthRouteImpl, IHealthRoute};
use axum::Router;

pub trait IRoute {
    fn setup(&self) -> Router;
}

pub struct RouteImpl {
    router: Router,
}

impl RouteImpl {
    pub fn new(router: Router) -> Self {
        RouteImpl { router }
    }
}

impl IRoute for RouteImpl {
    fn setup(&self) -> Router {
        let router = self.router.clone();
        let health_route = HealthRouteImpl::new("/health".to_string(), router);
        health_route.register()
    }
}
