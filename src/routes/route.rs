use axum::Router;

use crate::routes::health_route::health_routes;

pub fn route_setup() -> Router {
    Router::new().nest("/api/v1", health_routes())
}
