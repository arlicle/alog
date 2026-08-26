// HTTP server module
use crate::config::Config;
use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub async fn start_server(config: Config) -> Result<()> {
    let app = Router::new().fallback_service(ServeDir::new(config.output_dir));

    let addr = SocketAddr::new(config.server.host.parse()?, config.server.port);

    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;

    println!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
