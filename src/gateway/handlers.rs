use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::metrics::MetricsCollector;
use crate::routing::ServiceRegistry;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub prompt: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResponse {
    pub request_id: String,
    pub response: String,
    pub model_used: String,
    pub latency_ms: u64,
    pub tokens_generated: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub healthy: bool,
    pub models_online: usize,
    pub vram_usage_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_requests: u64,
    pub average_latency_ms: f32,
    pub throughput_rps: f32,
    pub cache_hit_rate: f32,
}

pub fn create_router(
    registry: Arc<ServiceRegistry>,
    metrics: Arc<MetricsCollector>,
) -> Router {
    Router::new()
        .route("/query", post(query_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/models", get(models_handler))
        .with_state((registry, metrics))
}

pub async fn query_handler(
    Json(_req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    // This is a simplified implementation
    // In production, this would use the router and memory manager
    Ok(Json(QueryResponse {
        request_id: uuid::Uuid::new_v4().to_string(),
        response: "Mock response".to_string(),
        model_used: "default-model".to_string(),
        latency_ms: 150,
        tokens_generated: 100,
    }))
}

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "operational".to_string(),
        healthy: true,
        models_online: 3,
        vram_usage_percent: 65.0,
    })
}

pub async fn metrics_handler() -> Json<MetricsResponse> {
    Json(MetricsResponse {
        total_requests: 1000,
        average_latency_ms: 125.5,
        throughput_rps: 8.0,
        cache_hit_rate: 0.72,
    })
}

pub async fn models_handler() -> Json<Vec<String>> {
    Json(vec![
        "tiny-llama-1.1b".to_string(),
        "phi-3-mini-3.8b".to_string(),
        "deepseek-coder-1.3b".to_string(),
        "mistral-7b".to_string(),
    ])
}
