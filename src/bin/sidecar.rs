use sovereign_orchestrator::inference::{InferenceEngine, ModelSidecar};
use std::sync::Arc;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Get configuration from environment
    let model_id = std::env::var("MODEL_ID").unwrap_or_else(|_| "default-model".to_string());
    let model_path = std::env::var("MODEL_PATH").unwrap_or_else(|_| "/models/default.gguf".to_string());
    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "default".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse()?;
    let parameters: usize = std::env::var("PARAMETERS")
        .unwrap_or_else(|_| "1300000000".to_string())
        .parse()?;

    tracing::info!(
        "Starting ModelWorker sidecar for {} ({}B parameters) on port {}",
        model_name,
        parameters / 1_000_000_000,
        port
    );

    // Create inference engine
    let engine = Arc::new(InferenceEngine::new(model_path, model_name, parameters));

    // Initialize the engine
    engine.initialize()?;

    // Create sidecar
    let sidecar = ModelSidecar::new(model_id, engine, port);

    // Start the gRPC server
    sidecar.start().await?;

    Ok(())
}
