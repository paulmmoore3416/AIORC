use crate::core::Result;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineBackend {
    Simulation,
    Ollama { endpoint: String },
    LlamaCpp, // Placeholder for future use
}

pub struct InferenceEngine {
    model_path: String,
    model_name: String,
    parameters: usize,
    backend: EngineBackend,
    context: Arc<Mutex<Option<InferenceContext>>>,
    http_client: reqwest::Client,
}

pub struct InferenceContext {
    _prompt_tokens: Vec<i32>,
    _completion_tokens: Vec<i32>,
    _total_tokens: usize,
}

impl InferenceEngine {
    pub fn new(model_path: String, model_name: String, parameters: usize, backend: EngineBackend) -> Self {
        InferenceEngine {
            model_path,
            model_name,
            parameters,
            backend,
            context: Arc::new(Mutex::new(None)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Initialize the inference engine (load model)
    pub fn initialize(&self) -> Result<()> {
        let mut context = self.context.lock();
        *context = Some(InferenceContext {
            _prompt_tokens: Vec::new(),
            _completion_tokens: Vec::new(),
            _total_tokens: 0,
        });

        tracing::info!(
            "Inference engine initialized for model: {} ({} parameters) with backend: {:?}",
            self.model_name,
            self.parameters,
            self.backend
        );

        Ok(())
    }

    /// Run inference on a prompt
    pub async fn infer(
        &self,
        prompt: &str,
        temperature: f32,
        max_tokens: i32,
    ) -> Result<Vec<String>> {
        {
            let context = self.context.lock();
            if context.is_none() {
                return Err(crate::core::OrchestratorError::InferenceError(
                    "Engine not initialized".to_string(),
                ));
            }
        }

        match &self.backend {
            EngineBackend::Simulation => {
                Ok(self.simulate_inference(prompt, max_tokens))
            }
            EngineBackend::Ollama { endpoint } => {
                self.infer_ollama(endpoint, prompt, temperature, max_tokens).await
            }
            EngineBackend::LlamaCpp => {
                Err(crate::core::OrchestratorError::InferenceError(
                    "LlamaCpp backend not yet implemented".to_string(),
                ))
            }
        }
    }

    async fn infer_ollama(
        &self,
        endpoint: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: i32,
    ) -> Result<Vec<String>> {
        let url = format!("{}/api/generate", endpoint);
        let body = serde_json::json!({
            "model": self.model_name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": temperature,
                "num_predict": max_tokens
            }
        });

        let response = self.http_client.post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::core::OrchestratorError::InferenceError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(crate::core::OrchestratorError::InferenceError(
                format!("Ollama error ({}): {}", status, err_text)
            ));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::core::OrchestratorError::InferenceError(e.to_string()))?;

        let response_text = json["response"].as_str()
            .ok_or_else(|| crate::core::OrchestratorError::InferenceError("Missing response field".to_string()))?;

        // Ollama returns a full string, but our internal trait expects tokens.
        // For simplicity, we split by characters or just return as one "token"
        Ok(vec![response_text.to_string()])
    }

    /// Stream inference with token-by-token output
    pub async fn infer_stream(
        &self,
        prompt: &str,
        temperature: f32,
        max_tokens: i32,
    ) -> Result<Vec<String>> {
        self.infer(prompt, temperature, max_tokens).await
    }

    /// Get model metadata
    pub fn get_metadata(&self) -> ModelMetadata {
        ModelMetadata {
            model_name: self.model_name.clone(),
            model_path: self.model_path.clone(),
            parameters: self.parameters,
            context_size: 2048,
            vocab_size: 32000,
        }
    }

    /// Simulate token generation (placeholder for real inference)
    fn simulate_inference(&self, _prompt: &str, max_tokens: i32) -> Vec<String> {
        let mut tokens = Vec::new();
        for i in 0..max_tokens.min(10) {
            tokens.push(format!("token_{}", i));
        }
        tokens
    }

    /// Clear context
    pub fn clear_context(&self) {
        let mut context = self.context.lock();
        *context = None;
    }

    /// Check if engine is ready
    pub fn is_ready(&self) -> bool {
        self.context.lock().is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub model_name: String,
    pub model_path: String,
    pub parameters: usize,
    pub context_size: usize,
    pub vocab_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_engine_creation() {
        let engine = InferenceEngine::new(
            "/models/test.gguf".to_string(),
            "test-model".to_string(),
            1000000,
            EngineBackend::Simulation,
        );
        assert_eq!(engine.model_name, "test-model");
        assert_eq!(engine.parameters, 1000000);
    }

    #[test]
    fn test_engine_initialization() {
        let engine = InferenceEngine::new(
            "/models/test.gguf".to_string(),
            "test-model".to_string(),
            1000000,
            EngineBackend::Simulation,
        );
        assert!(engine.initialize().is_ok());
        assert!(engine.is_ready());
    }

    #[test]
    fn test_ollama_backend_config() {
        let endpoint = "http://localhost:11434".to_string();
        let engine = InferenceEngine::new(
            "/models/test.gguf".to_string(),
            "test-model".to_string(),
            1000000,
            EngineBackend::Ollama { endpoint: endpoint.clone() },
        );
        if let EngineBackend::Ollama { endpoint: ep } = &engine.backend {
            assert_eq!(ep, &endpoint);
        } else {
            panic!("Backend should be Ollama");
        }
    }
}
