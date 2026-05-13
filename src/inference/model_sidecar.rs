use crate::core::Result as CoreResult;
use crate::orchestrator::model_worker_server::{ModelWorker, ModelWorkerServer};
use crate::orchestrator::{HealthCheckRequest, HealthCheckResponse, InferenceRequest, InferenceResponse, ModelInfoRequest, ModelInfoResponse};
use crate::inference::InferenceEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status, transport::Server};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct ModelSidecar {
    model_id: String,
    engine: Arc<InferenceEngine>,
    port: u16,
}

impl ModelSidecar {
    pub fn new(model_id: String, engine: Arc<InferenceEngine>, port: u16) -> Self {
        ModelSidecar {
            model_id,
            engine,
            port,
        }
    }

    /// Start the gRPC server
    pub async fn start(&self) -> CoreResult<()> {
        let addr = format!("0.0.0.0:{}", self.port).parse().map_err(|_| {
            crate::core::OrchestratorError::ConfigError("Invalid address".to_string())
        })?;

        let worker = ModelWorkerService {
            engine: Arc::clone(&self.engine),
            model_id: self.model_id.clone(),
        };

        tracing::info!(
            "Starting ModelWorker gRPC server for {} on {}",
            self.model_id,
            addr
        );

        Server::builder()
            .add_service(ModelWorkerServer::new(worker))
            .serve(addr)
            .await
            .map_err(|e| crate::core::OrchestratorError::GrpcError(
                Status::internal(e.to_string())
            ))
    }
}

pub struct ModelWorkerService {
    engine: Arc<InferenceEngine>,
    model_id: String,
}

#[tonic::async_trait]
impl ModelWorker for ModelWorkerService {
    type GenerateStream = ReceiverStream<Result<InferenceResponse, Status>>;

    async fn generate(
        &self,
        request: Request<InferenceRequest>,
    ) -> Result<Response<Self::GenerateStream>, Status> {
        let inner = request.into_inner();
        let engine = Arc::clone(&self.engine);
        let request_id = inner.request_id.clone();
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            match engine.infer(&inner.prompt, inner.temperature, inner.max_tokens) {
                Ok(tokens) => {
                    let total_tokens = tokens.len() as i32;
                    for (idx, token) in tokens.iter().enumerate() {
                        let is_final = idx == tokens.len() - 1;
                        let response = InferenceResponse {
                            token: token.clone(),
                            is_final,
                            confidence_score: 0.95,
                            request_id: request_id.clone(),
                            token_count: idx as i32 + 1,
                        };
                        let _ = tx.send(Ok(response)).await;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Status::internal(e.to_string()))).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let response = HealthCheckResponse {
            healthy: self.engine.is_ready(),
            vram_usage: 50.0,
            cpu_usage: 25.0,
            status: if self.engine.is_ready() {
                "healthy".to_string()
            } else {
                "initializing".to_string()
            },
        };
        Ok(Response::new(response))
    }

    async fn get_model_info(
        &self,
        _request: Request<ModelInfoRequest>,
    ) -> Result<Response<ModelInfoResponse>, Status> {
        let metadata = self.engine.get_metadata();
        let response = ModelInfoResponse {
            model_id: self.model_id.clone(),
            model_name: metadata.model_name,
            parameters: metadata.parameters as i32,
            architecture: "transformer".to_string(),
            vram_required: 2048,
            supported_tasks: vec![
                "chat".to_string(),
                "code".to_string(),
                "reasoning".to_string(),
            ],
            avg_latency_ms: 150.0,
        };
        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_creation() {
        let engine = Arc::new(InferenceEngine::new(
            "/models/test.gguf".to_string(),
            "test-model".to_string(),
            1000000,
        ));
        let sidecar = ModelSidecar::new("model-1".to_string(), engine, 50051);
        assert_eq!(sidecar.model_id, "model-1");
        assert_eq!(sidecar.port, 50051);
    }
}
