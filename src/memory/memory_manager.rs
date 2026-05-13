use crate::core::Result;
use crate::memory::{WarmSwapManager, SemanticCache};
use std::sync::Arc;

pub struct MemoryManager {
    warm_swap: Arc<WarmSwapManager>,
    semantic_cache: Arc<SemanticCache>,
    max_vram_mb: usize,
}

impl MemoryManager {
    pub fn new(
        max_vram_mb: usize,
        max_cache_size: usize,
        cache_similarity_threshold: f32,
    ) -> Result<Self> {
        Ok(MemoryManager {
            warm_swap: Arc::new(WarmSwapManager::new(max_vram_mb, max_cache_size)?),
            semantic_cache: Arc::new(SemanticCache::new(1000, cache_similarity_threshold)),
            max_vram_mb,
        })
    }

    /// Ensure a model is available in memory
    pub fn ensure_model_available(&self, model_id: &str, size_mb: usize) -> Result<u64> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.warm_swap.load_model(model_id, size_mb)?;

        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64 - start_time;

        Ok(elapsed)
    }

    /// Get the warm swap manager
    pub fn get_warm_swap(&self) -> Arc<WarmSwapManager> {
        Arc::clone(&self.warm_swap)
    }

    /// Get the semantic cache
    pub fn get_semantic_cache(&self) -> Arc<SemanticCache> {
        Arc::clone(&self.semantic_cache)
    }

    /// Get current VRAM usage
    pub fn get_vram_usage_mb(&self) -> usize {
        self.warm_swap.get_vram_usage_mb()
    }

    /// Get VRAM usage percentage
    pub fn get_vram_usage_percent(&self) -> f32 {
        self.warm_swap.get_vram_usage_percent()
    }

    /// Get list of loaded models
    pub fn get_loaded_models(&self) -> Vec<String> {
        self.warm_swap.get_loaded_models()
    }

    /// Health check
    pub fn is_healthy(&self) -> bool {
        self.get_vram_usage_percent() < 95.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new(4096, 3, 0.92).unwrap();
        assert_eq!(manager.max_vram_mb, 4096);
        assert!(manager.is_healthy());
    }
}
