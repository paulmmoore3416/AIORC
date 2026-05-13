use sovereign_orchestrator::core::Config;
use sovereign_orchestrator::gateway::OrchestratorGateway;
use std::path::Path;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load configuration
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.json".to_string());

    let config = if Path::new(&config_path).exists() {
        tracing::info!("Loading configuration from {}", config_path);
        Config::from_file(&config_path)?
    } else {
        tracing::info!("Using default configuration");
        Config::default()
    };

    // Create and start the gateway
    let gateway = OrchestratorGateway::new(config)?;
    gateway.start().await?;

    Ok(())
}
