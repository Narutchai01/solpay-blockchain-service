#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: &'static str,
}

pub async fn run_server() -> anyhow::Result<()> {
    let health_route = warp::path!("health").map(|| {
        let response = HealthResponse {
            status: "OK".to_string(),
            service: "solpay-blockchain-service".to_string(),
            version: env!("CARGO_PKG_VERSION"),
        };
        warp::reply::json(&response)
    });

    warp::serve(health_route).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
