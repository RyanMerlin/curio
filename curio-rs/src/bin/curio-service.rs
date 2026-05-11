use anyhow::Result;
use curio::service::{ServiceConfig, ServiceRuntime, serve};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = ServiceConfig::from_env()?;
    let runtime = ServiceRuntime::from_config(config.clone())?;
    serve(runtime, config.bind_addr).await
}
