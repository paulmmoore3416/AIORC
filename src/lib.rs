pub mod core;
pub mod routing;
pub mod memory;
pub mod inference;
pub mod gateway;

// Re-export proto-generated code
pub mod orchestrator {
    tonic::include_proto!("orchestrator");
}

pub use orchestrator::*;

// Version info
pub const VERSION: &str = "0.1.0";
pub const NAME: &str = "Sovereign Orchestrator";
