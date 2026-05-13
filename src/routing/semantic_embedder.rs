use std::collections::HashMap;

/// Lightweight semantic embedder using TF-IDF-like approach
pub struct SemanticEmbedder {
    task_vectors: HashMap<String, Vec<f32>>,
}

impl SemanticEmbedder {
    pub fn new() -> Self {
        let mut embedder = SemanticEmbedder {
            task_vectors: HashMap::new(),
        };
        embedder.initialize_task_vectors();
        embedder
    }

    /// Initialize predefined task vectors (centroids)
    fn initialize_task_vectors(&mut self) {
        // Task vectors are abstract representations of different task types
        // In production, these would be computed from actual model embeddings

        // Code-related tasks
        let mut code_vec = vec![0.0; 256];
        code_vec[0..20].iter_mut().for_each(|x| *x = 0.9); // Code-related features
        self.task_vectors.insert("code".to_string(), code_vec);

        // Creative writing tasks
        let mut creative_vec = vec![0.0; 256];
        creative_vec[20..40].iter_mut().for_each(|x| *x = 0.9); // Creative features
        self.task_vectors.insert("creative".to_string(), creative_vec);

        // Logic and reasoning
        let mut logic_vec = vec![0.0; 256];
        logic_vec[40..60].iter_mut().for_each(|x| *x = 0.9); // Logic features
        self.task_vectors.insert("logic".to_string(), logic_vec);

        // Summarization
        let mut summary_vec = vec![0.0; 256];
        summary_vec[60..80].iter_mut().for_each(|x| *x = 0.9); // Summary features
        self.task_vectors.insert("summarization".to_string(), summary_vec);

        // General conversation
        let mut chat_vec = vec![0.0; 256];
        chat_vec[80..100].iter_mut().for_each(|x| *x = 0.9); // Chat features
        self.task_vectors.insert("chat".to_string(), chat_vec);

        // Math and calculations
        let mut math_vec = vec![0.0; 256];
        math_vec[100..120].iter_mut().for_each(|x| *x = 0.9); // Math features
        self.task_vectors.insert("math".to_string(), math_vec);

        // Data extraction
        let mut data_vec = vec![0.0; 256];
        data_vec[120..140].iter_mut().for_each(|x| *x = 0.9); // Data features
        self.task_vectors.insert("data".to_string(), data_vec);
    }

    /// Generate a lightweight embedding for a prompt
    pub fn embed_prompt(&self, prompt: &str) -> Vec<f32> {
        let mut embedding = vec![0.0; 256];
        let lowercase_prompt = prompt.to_lowercase();

        // Keyword-based feature activation for mock
        if lowercase_prompt.contains("rust") || lowercase_prompt.contains("function") || lowercase_prompt.contains("code") {
            embedding[0..20].iter_mut().for_each(|x| *x = 0.8);
        }
        if lowercase_prompt.contains("calculate") || lowercase_prompt.contains("derivative") || lowercase_prompt.contains("math") {
            embedding[100..120].iter_mut().for_each(|x| *x = 0.8);
        }
        if lowercase_prompt.contains("story") || lowercase_prompt.contains("poem") {
            embedding[20..40].iter_mut().for_each(|x| *x = 0.8);
        }
        if lowercase_prompt.contains("reason") || lowercase_prompt.contains("logic") {
            embedding[40..60].iter_mut().for_each(|x| *x = 0.8);
        }
        if lowercase_prompt.contains("summarize") || lowercase_prompt.contains("summary") {
            embedding[60..80].iter_mut().for_each(|x| *x = 0.8);
        }

        // Add some noise based on words
        let words: Vec<&str> = lowercase_prompt
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        for word in words.iter() {
            let hash = self.hash_word(word) % 256;
            embedding[hash] = (embedding[hash] + 0.1_f32).min(1.0);
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }

        embedding
    }

    /// Find the most similar task based on semantic similarity
    pub fn find_most_similar_task(&self, prompt: &str) -> String {
        let prompt_embedding = self.embed_prompt(prompt);
        let mut best_task = "general".to_string();
        let mut best_similarity = -1.0f32;

        for (task_name, task_vec) in &self.task_vectors {
            let similarity = self.cosine_similarity(&prompt_embedding, task_vec);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_task = task_name.clone();
            }
        }

        best_task
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1 * norm2)
        } else {
            0.0
        }
    }

    /// Simple hash function for words
    fn hash_word(&self, word: &str) -> usize {
        word.bytes()
            .fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(b as usize))
    }

    /// Compare two embeddings for cache lookup
    pub fn similarity(&self, emb1: &[f32], emb2: &[f32]) -> f32 {
        if emb1.len() != emb2.len() {
            return 0.0;
        }
        self.cosine_similarity(emb1, emb2)
    }
}

impl Default for SemanticEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_classification() {
        let embedder = SemanticEmbedder::new();
        
        let code_prompt = "Write a Rust function";
        let code_task = embedder.find_most_similar_task(code_prompt);
        assert_eq!(code_task, "code");

        let math_prompt = "Calculate the derivative";
        let math_task = embedder.find_most_similar_task(math_prompt);
        assert_eq!(math_task, "math");
    }

    #[test]
    fn test_embedding_normalization() {
        let embedder = SemanticEmbedder::new();
        let embedding = embedder.embed_prompt("test prompt");
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
}
