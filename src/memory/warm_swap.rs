use crate::core::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ModelWeights {
    pub model_id: String,
    pub size_mb: usize,
    pub loaded_at: u64,
    pub last_accessed: u64,
}

pub struct WarmSwapManager {
    cache: Arc<Mutex<LruCache<String, ModelWeights>>>,
    total_vram_mb: usize,
    currently_loaded_mb: Arc<Mutex<usize>>,
    swap_history: Arc<Mutex<Vec<SwapEvent>>>,
}

#[derive(Debug, Clone)]
pub struct SwapEvent {
    pub timestamp: u64,
    pub model_id: String,
    pub action: SwapAction,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum SwapAction {
    Load,
    Unload,
    Reuse,
}

impl WarmSwapManager {
    pub fn new(total_vram_mb: usize, max_cache_size: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(max_cache_size).unwrap_or(NonZeroUsize::new(3).unwrap());
        Ok(WarmSwapManager {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            total_vram_mb,
            currently_loaded_mb: Arc::new(Mutex::new(0)),
            swap_history: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Attempt to load a model into VRAM
    pub fn load_model(&self, model_id: &str, size_mb: usize) -> Result<bool> {
        let start_time = self.get_current_time_ms();
        let mut cache = self.cache.lock();
        let mut loaded_mb = self.currently_loaded_mb.lock();

        // Check if already loaded
        if cache.contains(model_id) {
            if let Some(weights) = cache.get_mut(model_id) {
                weights.last_accessed = self.get_current_time_ms();
            }
            
            self.record_swap_event(
                model_id.to_string(),
                SwapAction::Reuse,
                self.get_current_time_ms() - start_time,
            );
            return Ok(true);
        }

        // Check if we have enough space
        if *loaded_mb + size_mb > self.total_vram_mb {
            // Evict the least recently used model
            if let Some((evicted_id, _)) = cache.pop_lru() {
                *loaded_mb = loaded_mb.saturating_sub(
                    cache.get(&evicted_id).map(|w| w.size_mb).unwrap_or(0),
                );
                self.record_swap_event(
                    evicted_id,
                    SwapAction::Unload,
                    self.get_current_time_ms() - start_time,
                );
            }
        }

        // Load the new model
        if *loaded_mb + size_mb <= self.total_vram_mb {
            let weights = ModelWeights {
                model_id: model_id.to_string(),
                size_mb,
                loaded_at: self.get_current_time_ms(),
                last_accessed: self.get_current_time_ms(),
            };
            cache.put(model_id.to_string(), weights);
            *loaded_mb += size_mb;
            
            self.record_swap_event(
                model_id.to_string(),
                SwapAction::Load,
                self.get_current_time_ms() - start_time,
            );
            Ok(true)
        } else {
            Err(crate::core::OrchestratorError::MemoryError(
                format!("Cannot load model {}: insufficient VRAM", model_id),
            ))
        }
    }

    /// Get the current VRAM usage
    pub fn get_vram_usage_mb(&self) -> usize {
        *self.currently_loaded_mb.lock()
    }

    /// Get VRAM usage percentage
    pub fn get_vram_usage_percent(&self) -> f32 {
        let usage = *self.currently_loaded_mb.lock();
        (usage as f32 / self.total_vram_mb as f32) * 100.0
    }

    /// Check if a model is currently loaded
    pub fn is_model_loaded(&self, model_id: &str) -> bool {
        self.cache.lock().contains(model_id)
    }

    /// Get list of loaded models
    pub fn get_loaded_models(&self) -> Vec<String> {
        self.cache
            .lock()
            .iter()
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get estimated swap time in milliseconds
    pub fn estimate_swap_time(&self, model_size_mb: usize) -> u64 {
        // Estimate based on NVMe speed (~500 MB/s for 4KB reads)
        // Rough calculation: 1ms per 500MB
        (model_size_mb as u64 / 500).max(10)
    }

    /// Record a swap event for monitoring
    fn record_swap_event(&self, model_id: String, action: SwapAction, latency_ms: u64) {
        let event = SwapEvent {
            timestamp: self.get_current_time_ms(),
            model_id,
            action,
            latency_ms,
        };
        let mut history = self.swap_history.lock();
        history.push(event);

        // Keep only last 1000 events
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get swap history
    pub fn get_swap_history(&self, limit: usize) -> Vec<SwapEvent> {
        self.swap_history
            .lock()
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get current time in milliseconds
    fn get_current_time_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Clear all loaded models
    pub fn clear(&self) {
        self.cache.lock().clear();
        *self.currently_loaded_mb.lock() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_loading() {
        let manager = WarmSwapManager::new(4096, 3).unwrap();
        assert!(manager.load_model("model1", 1024).is_ok());
        assert!(manager.is_model_loaded("model1"));
        assert_eq!(manager.get_vram_usage_mb(), 1024);
    }

    #[test]
    fn test_swap_estimation() {
        let manager = WarmSwapManager::new(4096, 3).unwrap();
        let latency = manager.estimate_swap_time(2048);
        assert!(latency > 0);
    }
}
