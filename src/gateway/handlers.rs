use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router as AxumRouter,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::metrics::MetricsCollector;
use crate::routing::{Router as OrchestratorRouter, ServiceRegistry};
use crate::memory::MemoryManager;

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

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<ServiceRegistry>,
    pub metrics: Arc<MetricsCollector>,
    pub router: Arc<OrchestratorRouter>,
    pub memory: Arc<MemoryManager>,
}

pub fn create_router(
    registry: Arc<ServiceRegistry>,
    metrics: Arc<MetricsCollector>,
    router: Arc<OrchestratorRouter>,
    memory: Arc<MemoryManager>,
) -> AxumRouter {
    AxumRouter::new()
        .route("/query", post(query_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/models", get(models_handler))
        .with_state(AppState {
            registry,
            metrics,
            router,
            memory,
        })
}

pub async fn query_handler(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    tracing::info!("Received query request: {}", req.prompt);

    // 1. Route the prompt
    let decision = state.router.route(&req.prompt).map_err(|e| {
        tracing::error!("Routing error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tracing::info!("Routed to model: {}", decision.target_model);

    // 2. Get model info
    let model_info = state.registry.get_model(&decision.target_model).ok_or_else(|| {
        tracing::error!("Model not found in registry: {}", decision.target_model);
        StatusCode::NOT_FOUND
    })?;

    // 3. Ensure model is available in VRAM
    state.memory.ensure_model_available(&model_info.model_id, model_info.vram_usage_mb as usize)
        .map_err(|e| {
            tracing::error!("Memory error for {}: {}", model_info.model_id, e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // 4. Call model worker with retry
    let mut attempts = 0;
    let max_attempts = 2;
    let mut last_error_msg = String::from("Unknown error");

    while attempts < max_attempts {
        let mut client = match crate::orchestrator::model_worker_client::ModelWorkerClient::connect(format!("http://127.0.0.1:{}", model_info.port)).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Retry {}/{}: Failed to connect to model {}: {}", attempts + 1, max_attempts, model_info.model_id, e);
                last_error_msg = e.to_string();
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
        };

        let grpc_request = tonic::Request::new(crate::orchestrator::InferenceRequest {
            prompt: req.prompt.clone(),
            temperature: req.temperature.unwrap_or(0.7),
            max_tokens: req.max_tokens.unwrap_or(256),
            top_p: "0.95".to_string(),
            metadata: std::collections::HashMap::new(),
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });

        match client.generate(grpc_request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut full_response = String::new();
                let mut tokens_generated = 0;

                while let Some(res) = tokio_stream::StreamExt::next(&mut stream).await {
                    let res = match res {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!("Stream error from {}: {}", model_info.model_id, e);
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    };
                    full_response.push_str(&res.token);
                    tokens_generated = res.token_count;
                    if res.is_final {
                        break;
                    }
                }

                let latency = start_time.elapsed().as_millis() as u64;
                
                // Update metrics
                state.metrics.record_request(crate::core::RequestMetrics {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    routed_model: decision.target_model.clone(),
                    complexity_score: decision.complexity_score,
                    tokens_generated,
                    latency_ms: latency,
                    confidence: decision.confidence,
                });

                return Ok(Json(QueryResponse {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    response: full_response,
                    model_used: decision.target_model,
                    latency_ms: latency,
                    tokens_generated,
                }));
            }
            Err(e) => {
                tracing::warn!("Retry {}/{}: gRPC error from {}: {}", attempts + 1, max_attempts, model_info.model_id, e);
                last_error_msg = e.to_string();
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    tracing::error!("All {} attempts failed for {}: {}", max_attempts, model_info.model_id, last_error_msg);
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn health_handler(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "operational".to_string(),
        healthy: state.memory.is_healthy(),
        models_online: state.registry.healthy_model_count(),
        vram_usage_percent: state.memory.get_vram_usage_percent(),
    })
}

pub async fn metrics_handler(
    State(state): State<AppState>,
) -> Json<MetricsResponse> {
    let metrics = state.metrics.get_system_metrics();
    Json(MetricsResponse {
        total_requests: metrics.total_requests,
        average_latency_ms: metrics.average_latency_ms,
        throughput_rps: metrics.throughput_rps,
        cache_hit_rate: metrics.cache_hit_rate,
    })
}

pub async fn models_handler(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let models = state.registry.list_all_models();
    Json(models.into_iter().map(|m| m.model_id).collect())
}
