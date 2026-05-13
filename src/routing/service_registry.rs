use crate::core::types::{ModelWorkerInfo, WorkerStatus};
use dashmap::DashMap;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct ServiceRegistry {
    models: Arc<DashMap<String, ModelWorkerInfo>>,
    healthy_models_cache: Arc<Mutex<Vec<String>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        ServiceRegistry {
            models: Arc::new(DashMap::new()),
            healthy_models_cache: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a new model worker
    pub fn register(&self, model: ModelWorkerInfo) -> crate::core::Result<()> {
        if self.models.contains_key(&model.model_id) {
            return Err(crate::core::OrchestratorError::InternalError(format!(
                "Model {} already registered",
                model.model_id
            )));
        }
        self.models.insert(model.model_id.clone(), model);
        self.invalidate_cache();
        Ok(())
    }

    /// Deregister a model worker
    pub fn deregister(&self, model_id: &str) -> Result<(), String> {
        if self.models.remove(model_id).is_none() {
            return Err(format!("Model {} not found", model_id));
        }
        self.invalidate_cache();
        Ok(())
    }

    /// Update model status
    pub fn update_status(&self, model_id: &str, status: WorkerStatus) -> Result<(), String> {
        if let Some(mut model) = self.models.get_mut(model_id) {
            model.status = status;
            self.invalidate_cache();
            Ok(())
        } else {
            Err(format!("Model {} not found", model_id))
        }
    }

    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<ModelWorkerInfo> {
        self.models.get(model_id).map(|entry| entry.clone())
    }

    /// Get all healthy models
    pub fn get_healthy_models(&self) -> Vec<ModelWorkerInfo> {
        self.models
            .iter()
            .filter(|entry| entry.status == WorkerStatus::Healthy)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get models supporting a specific task
    pub fn get_models_by_task(&self, task: &str) -> Vec<ModelWorkerInfo> {
        self.models
            .iter()
            .filter(|entry| {
                entry.status == WorkerStatus::Healthy
                    && entry.supported_tasks.iter().any(|t| t == task)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get the least loaded model
    pub fn get_least_loaded_model(&self) -> Option<ModelWorkerInfo> {
        self.models
            .iter()
            .filter(|entry| entry.status == WorkerStatus::Healthy)
            .min_by(|a, b| {
                a.value()
                    .current_load
                    .partial_cmp(&b.value().current_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|entry| entry.value().clone())
    }

    /// Get all registered models
    pub fn list_all_models(&self) -> Vec<ModelWorkerInfo> {
        self.models.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Check if a model exists and is healthy
    pub fn is_healthy(&self, model_id: &str) -> bool {
        self.models
            .get(model_id)
            .map(|m| m.status == WorkerStatus::Healthy)
            .unwrap_or(false)
    }

    /// Get model count
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Get healthy model count
    pub fn healthy_model_count(&self) -> usize {
        self.models
            .iter()
            .filter(|entry| entry.status == WorkerStatus::Healthy)
            .count()
    }

    /// Invalidate the cache of healthy models
    fn invalidate_cache(&self) {
        let mut cache = self.healthy_models_cache.lock();
        cache.clear();
    }

    /// Clear all registrations
    pub fn clear(&self) {
        self.models.clear();
        self.invalidate_cache();
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registration() {
        let registry = ServiceRegistry::new();
        let model = ModelWorkerInfo {
            model_id: "test-model".to_string(),
            address: "localhost".to_string(),
            port: 50051,
            status: WorkerStatus::Healthy,
            supported_tasks: vec!["code".to_string()],
            vram_usage_mb: 2048.0,
            current_load: 0.5,
        };

        assert!(registry.register(model.clone()).is_ok());
        assert!(registry.get_model("test-model").is_some());
        assert_eq!(registry.model_count(), 1);
    }

    #[test]
    fn test_get_least_loaded() {
        let registry = ServiceRegistry::new();

        let model1 = ModelWorkerInfo {
            model_id: "model1".to_string(),
            address: "localhost".to_string(),
            port: 50051,
            status: WorkerStatus::Healthy,
            supported_tasks: vec![],
            vram_usage_mb: 1024.0,
            current_load: 0.8,
        };

        let model2 = ModelWorkerInfo {
            model_id: "model2".to_string(),
            address: "localhost".to_string(),
            port: 50052,
            status: WorkerStatus::Healthy,
            supported_tasks: vec![],
            vram_usage_mb: 1024.0,
            current_load: 0.2,
        };

        registry.register(model1).unwrap();
        registry.register(model2).unwrap();

        let least_loaded = registry.get_least_loaded_model().unwrap();
        assert_eq!(least_loaded.model_id, "model2");
    }
}
