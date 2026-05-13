use crate::core::types::CachedLogicChain;
use crate::core::Result;
use dashmap::DashMap;
use std::sync::Arc;
use sha2::{Sha256, Digest};

pub struct SemanticCache {
    cache: Arc<DashMap<String, CachedLogicChain>>,
    similarity_threshold: f32,
    max_cache_size: usize,
}

impl SemanticCache {
    pub fn new(max_cache_size: usize, similarity_threshold: f32) -> Self {
        SemanticCache {
            cache: Arc::new(DashMap::new()),
            similarity_threshold,
            max_cache_size,
        }
    }

    /// Hash a prompt for cache lookup
    pub fn hash_prompt(prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Store a logic chain in cache
    pub fn store(&self, chain: CachedLogicChain) -> Result<()> {
        if self.cache.len() >= self.max_cache_size {
            // Evict the least recently used entry
            if let Some(entry) = self
                .cache
                .iter()
                .min_by_key(|entry| entry.value().created_at)
            {
                let key = entry.key().clone();
                drop(entry);
                self.cache.remove(&key);
            }
        }

        self.cache.insert(chain.prompt_hash.clone(), chain);
        Ok(())
    }

    /// Retrieve a cached logic chain
    pub fn retrieve(&self, prompt_hash: &str) -> Option<CachedLogicChain> {
        if let Some(mut entry) = self.cache.get_mut(prompt_hash) {
            entry.hit_count += 1;
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Find similar cached chains (approximate semantic match)
    pub fn find_similar(&self, prompt: &str) -> Vec<CachedLogicChain> {
        let prompt_tokens: Vec<&str> = prompt.split_whitespace().collect();
        let mut results = Vec::new();

        for entry in self.cache.iter() {
            let cached_prompt_tokens: Vec<&str> = entry.value().id.split_whitespace().collect();
            
            // Simple Jaccard similarity
            let intersection = prompt_tokens
                .iter()
                .filter(|token| cached_prompt_tokens.contains(token))
                .count();
            let union = prompt_tokens.len() + cached_prompt_tokens.len() - intersection;
            let similarity = if union > 0 {
                intersection as f32 / union as f32
            } else {
                0.0
            };

            if similarity >= self.similarity_threshold {
                results.push(entry.value().clone());
            }
        }

        // Sort by hit count (most popular first)
        results.sort_by(|a, b| b.hit_count.cmp(&a.hit_count));
        results
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let total_hits: usize = self.cache.iter().map(|e| e.value().hit_count).sum();
        
        CacheStats {
            total_entries,
            max_size: self.max_cache_size,
            total_hits,
            average_confidence: if total_entries > 0 {
                self.cache.iter().map(|e| e.value().confidence).sum::<f32>() / total_entries as f32
            } else {
                0.0
            },
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Evict entries by age
    pub fn evict_old_entries(&self, age_seconds: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(age_seconds as i64);
        self.cache.retain(|_, v| v.created_at > cutoff);
    }

    /// Evict the least recently used entry
    pub fn evict_lru(&self) {
        if let Some(ref_multi) = self
            .cache
            .iter()
            .min_by_key(|entry| entry.value().created_at)
        {
            let key = ref_multi.key().clone();
            drop(ref_multi);
            self.cache.remove(&key);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub max_size: usize,
    pub total_hits: usize,
    pub average_confidence: f32,
}

impl Default for SemanticCache {
    fn default() -> Self {
        Self::new(1000, 0.92)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_prompt_hashing() {
        let hash1 = SemanticCache::hash_prompt("test prompt");
        let hash2 = SemanticCache::hash_prompt("test prompt");
        assert_eq!(hash1, hash2);

        let hash3 = SemanticCache::hash_prompt("different prompt");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_storage_and_retrieval() {
        let cache = SemanticCache::new(100, 0.92);
        let chain = CachedLogicChain {
            id: "test-chain".to_string(),
            prompt_hash: SemanticCache::hash_prompt("test"),
            task_type: "logic".to_string(),
            reasoning_steps: vec!["step1".to_string()],
            final_answer: "answer".to_string(),
            confidence: 0.95,
            created_at: Utc::now(),
            hit_count: 0,
        };

        cache.store(chain.clone()).unwrap();
        let retrieved = cache.retrieve(&chain.prompt_hash);
        assert!(retrieved.is_some());
    }
}
