#![allow(async_fn_in_trait)]
use crate::routes::route::{IRoute, RouteImpl};
use axum::{Router, routing::get};
use std::net::SocketAddr;

pub trait IServer {
    async fn start(&self);
}

pub struct ServerImpl {
    port: i32,
}

impl ServerImpl {
    pub fn new(port: i32) -> Self {
        ServerImpl { port }
    }
}
impl IServer for ServerImpl {
    async fn start(&self) {
        let mut app = Router::new().route(
            "/",
            get(|| async { "Welcome to Solpay Blockchain Service!" }),
        );

        let route_manager = RouteImpl::new(app);
        app = route_manager.setup();

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port as u16));
        println!("Listening on {}", addr);

        // 3. รัน Server
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
