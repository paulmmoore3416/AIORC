use crate::core::{Config, MetricsCollector, Result};
use crate::routing::{Router, ServiceRegistry};
use crate::memory::MemoryManager;
use crate::gateway::handlers::create_router as create_http_router;
use std::sync::Arc;
use axum::Router as AxumRouter;
use tracing::Level;

pub struct OrchestratorGateway {
    config: Config,
    router: Router,
    service_registry: Arc<ServiceRegistry>,
    memory_manager: Arc<MemoryManager>,
    metrics_collector: Arc<MetricsCollector>,
}

impl OrchestratorGateway {
    pub fn new(config: Config) -> Result<Self> {
        let service_registry = Arc::new(ServiceRegistry::new());
        
        // Register models from config
        for model_config in &config.models {
            let info = crate::core::types::ModelWorkerInfo {
                model_id: model_config.id.clone(),
                address: "127.0.0.1".to_string(), // Default to localhost for now
                port: model_config.port,
                status: crate::core::types::WorkerStatus::Healthy,
                supported_tasks: model_config.supported_tasks.clone(),
                vram_usage_mb: model_config.vram_required_mb as f32,
                current_load: 0.0,
            };
            service_registry.register(info)?;
        }

        let router = Router::new(Arc::clone(&service_registry));
        
        let memory_manager = Arc::new(MemoryManager::new(
            config.memory.max_vram_mb,
            config.memory.model_cache_size,
            config.router.cache_similarity_threshold,
        )?);

        let metrics_collector = Arc::new(MetricsCollector::new());

        Ok(OrchestratorGateway {
            config,
            router,
            service_registry,
            memory_manager,
            metrics_collector,
        })
    }

    /// Start the gateway server
    pub async fn start(&self) -> Result<()> {
        // Initialize tracing
        self.init_tracing();

        tracing::info!(
            "Starting {} v{}",
            self.config.orchestrator.name,
            self.config.orchestrator.version
        );

        // Start HTTP server
        self.start_http_server().await?;

        Ok(())
    }

    /// Start the HTTP gateway server
    async fn start_http_server(&self) -> Result<()> {
        let addr: std::net::SocketAddr = format!(
            "{}:{}",
            self.config.gateway.host, self.config.gateway.port
        )
        .parse()
        .map_err(|e: std::net::AddrParseError| {
            crate::core::OrchestratorError::ConfigError(format!("Invalid gateway address: {}", e))
        })?;

        let app = create_http_router(
            Arc::clone(&self.service_registry),
            Arc::clone(&self.metrics_collector),
        );

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| crate::core::OrchestratorError::IoError(e))?;

        tracing::info!("HTTP Gateway listening on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::core::OrchestratorError::CommunicationError(e.to_string()))
    }

    /// Initialize tracing for logging
    fn init_tracing(&self) {
        let _subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .init();
    }

    /// Get a reference to the router
    pub fn get_router(&self) -> &Router {
        &self.router
    }

    /// Get a reference to the service registry
    pub fn get_registry(&self) -> Arc<ServiceRegistry> {
        Arc::clone(&self.service_registry)
    }

    /// Get a reference to the memory manager
    pub fn get_memory_manager(&self) -> Arc<MemoryManager> {
        Arc::clone(&self.memory_manager)
    }

    /// Get a reference to the metrics collector
    pub fn get_metrics(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        let config = Config::default();
        let gateway = OrchestratorGateway::new(config);
        assert!(gateway.is_ok());
    }
}
