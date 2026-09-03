mod api;
mod model;
mod route;
mod weather;

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use api::{router, AppState};
use tokio::sync::RwLock;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let address = std::env::var("WEATHER_LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("WEATHER_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse::<u16>()
        .context("WEATHER_PORT must be a valid port")?;
    let static_dir =
        PathBuf::from(std::env::var("WEATHER_STATIC_DIR").unwrap_or_else(|_| "static".into()));
    let sample_km = std::env::var("WEATHER_SAMPLE_KM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;
    let state = Arc::new(AppState {
        trips: RwLock::new(Default::default()),
        client,
        sample_km,
    });
    let app = router(state, static_dir);
    let bind: SocketAddr = format!("{address}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    info!(%bind, "serving Alpine Weather Route");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
