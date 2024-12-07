use lightning_terminal_gateway::{GatewayConfig, build_router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = GatewayConfig::from_env()?;
    let listener = TcpListener::bind(config.bind_addr).await?;

    info!(
        bind_addr = %config.bind_addr,
        device_id = %config.device_id,
        "starting Lightning terminal gateway"
    );

    axum::serve(listener, build_router(config)).await?;
    Ok(())
}
