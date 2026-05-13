use crate::core::{OrchestratorError, Result, RoutingDecision};
use crate::routing::{ComplexityScorer, SemanticEmbedder, ServiceRegistry};
use std::sync::Arc;

pub struct Router {
    semantic_embedder: SemanticEmbedder,
    service_registry: Arc<ServiceRegistry>,
    consensus_threshold: f32,
}

impl Router {
    pub fn new(service_registry: Arc<ServiceRegistry>) -> Self {
        Router {
            semantic_embedder: SemanticEmbedder::new(),
            service_registry,
            consensus_threshold: 0.85,
        }
    }

    /// Make a routing decision based on the prompt
    pub fn route(&self, prompt: &str) -> Result<RoutingDecision> {
        // Step 1: Calculate complexity
        let complexity_score = ComplexityScorer::calculate_complexity(prompt);

        // Step 2: Determine task type via semantic analysis
        let task_type = self.semantic_embedder.find_most_similar_task(prompt);

        // Step 3: Get available models for this task
        let available_models = self.service_registry.get_models_by_task(&task_type);
        if available_models.is_empty() {
            // Fallback to least loaded healthy model
            let fallback = self
                .service_registry
                .get_least_loaded_model()
                .ok_or_else(|| OrchestratorError::ModelNotFound("No healthy models available".to_string()))?;
            let model_id = fallback.model_id.clone();
            return Ok(RoutingDecision {
                target_model: model_id.clone(),
                complexity_score,
                confidence: 0.5,
                reason: format!("Fallback to least loaded model: {}", model_id),
                use_consensus: ComplexityScorer::requires_consensus(complexity_score, 7),
            });
        }

        // Step 4: Select best model based on complexity
        let target_model = self.select_best_model(&available_models, complexity_score)?;

        // Step 5: Determine if consensus mode is needed
        let use_consensus = ComplexityScorer::requires_consensus(complexity_score, 7);

        Ok(RoutingDecision {
            target_model: target_model.model_id,
            complexity_score,
            confidence: 0.95,
            reason: format!(
                "Task: {}, Complexity: {}, Tier: {}",
                task_type,
                complexity_score,
                ComplexityScorer::get_tier(complexity_score)
            ),
            use_consensus,
        })
    }

    /// Select the best model from available candidates
    fn select_best_model(&self, models: &[crate::core::types::ModelWorkerInfo], complexity: u8) -> Result<crate::core::types::ModelWorkerInfo> {
        let tier = ComplexityScorer::get_tier(complexity);

        // For simple tasks, prefer smaller, faster models
        if tier == "instant" {
            models
                .iter()
                .min_by(|a, b| {
                    a.current_load
                        .partial_cmp(&b.current_load)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned()
                .ok_or_else(|| OrchestratorError::ModelNotFound("No models available".to_string()))
        } else {
            // For complex tasks, prefer larger models with lower load
            models
                .iter()
                .max_by(|a, b| {
                    let a_score = (1.0 - a.current_load) * 0.7; // Prefer lower load
                    let b_score = (1.0 - b.current_load) * 0.7;
                    a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned()
                .ok_or_else(|| OrchestratorError::ModelNotFound("No models available".to_string()))
        }
    }

    /// Fan-out query to multiple models (for consensus)
    pub fn route_consensus(&self, prompt: &str) -> Result<Vec<RoutingDecision>> {
        let healthy_models = self.service_registry.get_healthy_models();
        if healthy_models.len() < 2 {
            return Err(OrchestratorError::ModelNotFound(
                "Not enough healthy models for consensus".to_string(),
            ));
        }

        let complexity_score = ComplexityScorer::calculate_complexity(prompt);
        let decisions = healthy_models
            .into_iter()
            .take(2) // Use top 2 models for consensus
            .map(|model| RoutingDecision {
                target_model: model.model_id,
                complexity_score,
                confidence: 0.9,
                reason: format!("Consensus mode for prompt"),
                use_consensus: false,
            })
            .collect();

        Ok(decisions)
    }

    /// Set consensus threshold
    pub fn set_consensus_threshold(&mut self, threshold: f32) {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::WorkerStatus;

    #[test]
    fn test_routing_decision() {
        let registry = Arc::new(ServiceRegistry::new());
        let model = crate::core::types::ModelWorkerInfo {
            model_id: "test-model".to_string(),
            address: "localhost".to_string(),
            port: 50051,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["general".to_string()],
            vram_usage_mb: 1024.0,
            current_load: 0.5,
        };
        registry.register(model).unwrap();

        let router = Router::new(registry);
        let decision = router.route("Hello, how are you?").unwrap();
        assert_eq!(decision.complexity_score, 1);
    }
}
