use tracing::info;

use crate::routes::route_setup;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = route_setup();

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        info!("Server listening on {}", self.addr);

        axum::serve(listener, app).await?;

        Ok(())
    }
}
