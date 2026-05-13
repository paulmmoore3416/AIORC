use crate::core::types::{SystemMetrics, RequestMetrics};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use chrono::Utc;

#[derive(Clone)]
pub struct MetricsCollector {
    total_requests: Arc<AtomicU64>,
    total_latency_ms: Arc<Mutex<u64>>,
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    model_switches: Arc<AtomicU64>,
    request_history: Arc<DashMap<String, RequestMetrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_latency_ms: Arc::new(Mutex::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            model_switches: Arc::new(AtomicU64::new(0)),
            request_history: Arc::new(DashMap::new()),
        }
    }

    pub fn record_request(&self, metrics: RequestMetrics) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        *self.total_latency_ms.lock() += metrics.latency_ms;
        self.request_history.insert(metrics.request_id.clone(), metrics);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_model_switch(&self) {
        self.model_switches.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_system_metrics(&self) -> SystemMetrics {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_latency = *self.total_latency_ms.lock();
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let cache_total = cache_hits + cache_misses;

        let average_latency = if total_requests > 0 {
            (total_latency as f32) / (total_requests as f32)
        } else {
            0.0
        };

        let cache_hit_rate = if cache_total > 0 {
            (cache_hits as f32) / (cache_total as f32)
        } else {
            0.0
        };

        let vram_usage = 0.0; // Still hard to get without nvidia-smi bindings
        
        let memory_usage = if let Some(usage) = memory_stats::memory_stats() {
            (usage.physical_mem as f32 / (1024.0 * 1024.0 * 1024.0)) * 100.0 // Normalize as needed
        } else {
            0.0
        };

        SystemMetrics {
            total_requests,
            average_latency_ms: average_latency,
            throughput_rps: if average_latency > 0.0 { 1000.0 / average_latency } else { 0.0 },
            vram_usage_percent: vram_usage,
            cpu_usage_percent: 0.0, // Placeholder
            memory_usage_percent: memory_usage.min(100.0),
            active_models: 0,
            model_switch_count: self.model_switches.load(Ordering::Relaxed),
            cache_hit_rate,
        }
    }

    pub fn get_request_history(&self, limit: usize) -> Vec<RequestMetrics> {
        self.request_history
            .iter()
            .take(limit)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn clear_old_history(&self, hours: u64) {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        self.request_history.retain(|_, v| v.timestamp > cutoff);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
