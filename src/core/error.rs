use thiserror::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, OrchestratorError>;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Routing error: {0}")]
    RoutingError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Inference error: {0}")]
    InferenceError(String),

    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Timeout occurred")]
    Timeout,

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<OrchestratorError> for tonic::Status {
    fn from(err: OrchestratorError) -> tonic::Status {
        match err {
            OrchestratorError::ModelNotFound(msg) => {
                tonic::Status::not_found(msg)
            }
            OrchestratorError::RoutingError(msg) => {
                tonic::Status::internal(format!("Routing error: {}", msg))
            }
            OrchestratorError::MemoryError(msg) => {
                tonic::Status::resource_exhausted(msg)
            }
            OrchestratorError::InferenceError(msg) => {
                tonic::Status::internal(format!("Inference error: {}", msg))
            }
            OrchestratorError::GrpcError(status) => status,
            OrchestratorError::CommunicationError(msg) => {
                tonic::Status::unavailable(msg)
            }
            OrchestratorError::ConfigError(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            OrchestratorError::Timeout => {
                tonic::Status::deadline_exceeded("Request timeout")
            }
            OrchestratorError::ResourceExhausted(msg) => {
                tonic::Status::resource_exhausted(msg)
            }
            OrchestratorError::InvalidRequest(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
