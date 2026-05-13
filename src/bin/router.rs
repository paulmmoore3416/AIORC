use sovereign_orchestrator::core::types::{ModelWorkerInfo, WorkerStatus};
use sovereign_orchestrator::routing::Router;
use std::sync::Arc;
use tracing::Level;
use sovereign_orchestrator::routing::ServiceRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    tracing::info!("Starting Routing Engine...");

    // Create service registry and register mock models
    let registry = Arc::new(ServiceRegistry::new());

    // Register models
    let models = vec![
        ModelWorkerInfo {
            model_id: "tinyllama-1.1b".to_string(),
            address: "localhost".to_string(),
            port: 50051,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["chat".to_string(), "general".to_string()],
            vram_usage_mb: 512.0,
            current_load: 0.2,
        },
        ModelWorkerInfo {
            model_id: "phi-3-mini-3.8b".to_string(),
            address: "localhost".to_string(),
            port: 50052,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["reasoning".to_string(), "summarization".to_string()],
            vram_usage_mb: 2048.0,
            current_load: 0.5,
        },
        ModelWorkerInfo {
            model_id: "deepseek-coder-1.3b".to_string(),
            address: "localhost".to_string(),
            port: 50053,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["code".to_string(), "logic".to_string()],
            vram_usage_mb: 1024.0,
            current_load: 0.3,
        },
        ModelWorkerInfo {
            model_id: "mistral-7b".to_string(),
            address: "localhost".to_string(),
            port: 50054,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["expert".to_string(), "analysis".to_string(), "code".to_string()],
            vram_usage_mb: 4096.0,
            current_load: 0.8,
        },
    ];

    for model in models {
        registry.register(model)?;
    }

    tracing::info!("Registered {} models", registry.model_count());

    // Create router
    let router = Router::new(registry.clone());

    // Test prompts to demonstrate routing
    let test_prompts = vec![
        "Hello, how are you?",
        "Write a Rust function that implements quicksort",
        "Calculate the derivative of x^3 + 2x^2 - 5x + 3",
        "Analyze the following dataset and provide insights",
        "Write a poem about the beauty of mountains",
        "Explain the concept of machine learning in simple terms",
        "if condition then action else alternative reasoning inference deduction",
        "This is a very long prompt with extensive vocabulary and structural complexity. It requires deep reasoning and analysis. The task involves multiple steps: first, understand the context; second, decompose the problem; third, identify key relationships; fourth, synthesize a comprehensive solution; fifth, validate against edge cases; finally, present the results in a clear and concise manner.",
    ];

    tracing::info!("\n=== Routing Decisions Demo ===\n");

    for (idx, prompt) in test_prompts.iter().enumerate() {
        match router.route(prompt) {
            Ok(decision) => {
                tracing::info!(
                    "[{}] Prompt: \"{}\"",
                    idx + 1,
                    if prompt.len() > 50 {
                        format!("{}...", &prompt[..47])
                    } else {
                        prompt.to_string()
                    }
                );
                tracing::info!(
                    "    ├─ Complexity: {}/10 ({})",
                    decision.complexity_score,
                    sovereign_orchestrator::routing::ComplexityScorer::get_tier(decision.complexity_score)
                );
                tracing::info!("    ├─ Target Model: {}", decision.target_model);
                tracing::info!("    ├─ Confidence: {:.1}%", decision.confidence * 100.0);
                tracing::info!("    ├─ Use Consensus: {}", decision.use_consensus);
                tracing::info!("    └─ Reason: {}", decision.reason);
            }
            Err(e) => {
                tracing::error!("[{}] Routing error: {}", idx + 1, e);
            }
        }
        tracing::info!("");
    }

    tracing::info!("=== Registry Status ===");
    tracing::info!("Total Models: {}", registry.model_count());
    tracing::info!("Healthy Models: {}", registry.healthy_model_count());
    
    for model in registry.list_all_models() {
        tracing::info!(
            "  • {} ({}B params) - Load: {:.1}%",
            model.model_id,
            model.vram_usage_mb / 512.0,
            model.current_load * 100.0
        );
    }

    Ok(())
}
