use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub orchestrator: OrchestratorConfig,
    pub router: RouterConfig,
    pub memory: MemoryConfig,
    pub models: Vec<ModelConfig>,
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub name: String,
    pub version: String,
    pub max_concurrent_requests: usize,
    pub request_timeout_ms: u64,
    pub enable_metrics: bool,
    pub metrics_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub embedding_model: String,
    pub complexity_threshold_low: u8,
    pub complexity_threshold_high: u8,
    pub enable_consensus_mode: bool,
    pub consensus_threshold: f32,
    pub enable_semantic_cache: bool,
    pub cache_similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_vram_mb: usize,
    pub max_ram_mb: usize,
    pub enable_warm_swap: bool,
    pub swap_timeout_ms: u64,
    pub nvme_swap_path: Option<PathBuf>,
    pub model_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub model_path: PathBuf,
    pub model_type: ModelType,
    pub parameters: usize,
    pub vram_required_mb: usize,
    pub port: u16,
    pub supported_tasks: Vec<String>,
    pub quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "logic")]
    Logic,
    #[serde(rename = "summarization")]
    Summarization,
    #[serde(rename = "general")]
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub max_batch_size: usize,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            orchestrator: OrchestratorConfig {
                name: "Sovereign Orchestrator".to_string(),
                version: "0.1.0".to_string(),
                max_concurrent_requests: 100,
                request_timeout_ms: 30000,
                enable_metrics: true,
                metrics_interval_ms: 5000,
            },
            router: RouterConfig {
                embedding_model: "all-MiniLM-L6-v2".to_string(),
                complexity_threshold_low: 3,
                complexity_threshold_high: 7,
                enable_consensus_mode: true,
                consensus_threshold: 0.85,
                enable_semantic_cache: true,
                cache_similarity_threshold: 0.92,
            },
            memory: MemoryConfig {
                max_vram_mb: 4096,
                max_ram_mb: 8192,
                enable_warm_swap: true,
                swap_timeout_ms: 400,
                nvme_swap_path: Some(PathBuf::from("/tmp/model_swap")),
                model_cache_size: 3,
            },
            models: vec![],
            gateway: GatewayConfig {
                host: "0.0.0.0".to_string(),
                port: 9090,
                max_batch_size: 32,
                enable_cors: true,
                allowed_origins: vec!["*".to_string()],
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> crate::core::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::core::OrchestratorError::ConfigError(e.to_string()))?;
        let config = serde_json::from_str(&content)
            .map_err(|e| crate::core::OrchestratorError::ConfigError(e.to_string()))?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> crate::core::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| crate::core::OrchestratorError::ConfigError(e.to_string()))?;
        std::fs::write(path, json)
            .map_err(|e| crate::core::OrchestratorError::ConfigError(e.to_string()))?;
        Ok(())
    }
}
