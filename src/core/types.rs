use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub token: String,
    pub confidence: f32,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWorkerInfo {
    pub model_id: String,
    pub address: String,
    pub port: u16,
    pub status: WorkerStatus,
    pub supported_tasks: Vec<String>,
    pub vram_usage_mb: f32,
    pub current_load: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "loading")]
    Loading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub target_model: String,
    pub complexity_score: u8,
    pub confidence: f32,
    pub reason: String,
    pub use_consensus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub routed_model: String,
    pub complexity_score: u8,
    pub tokens_generated: i32,
    pub latency_ms: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVector {
    pub task_name: String,
    pub vector: Vec<f32>,
    pub centroid_position: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Logic,
    CreativeWriting,
    Code,
    DataExtraction,
    Summarization,
    GeneralChat,
    Math,
    Analysis,
}

impl TaskType {
    pub fn to_string(&self) -> String {
        match self {
            TaskType::Logic => "Logic".to_string(),
            TaskType::CreativeWriting => "CreativeWriting".to_string(),
            TaskType::Code => "Code".to_string(),
            TaskType::DataExtraction => "DataExtraction".to_string(),
            TaskType::Summarization => "Summarization".to_string(),
            TaskType::GeneralChat => "GeneralChat".to_string(),
            TaskType::Math => "Math".to_string(),
            TaskType::Analysis => "Analysis".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLogicChain {
    pub id: String,
    pub prompt_hash: String,
    pub task_type: String,
    pub reasoning_steps: Vec<String>,
    pub final_answer: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub hit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_requests: u64,
    pub average_latency_ms: f32,
    pub throughput_rps: f32,
    pub vram_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub active_models: usize,
    pub model_switch_count: u64,
    pub cache_hit_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub vram_usage_mb: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub uptime_seconds: u64,
    pub active_requests: usize,
}
