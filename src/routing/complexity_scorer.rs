use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref CODE_PATTERN: Regex = Regex::new(r"(?i)(fn |impl |class |def |function |var |let |const |import |from |=>|async |await)").unwrap();
    static ref MATH_PATTERN: Regex = Regex::new(r"[0-9]+\s*[\+\-\*/]\s*[0-9]+|∑|∏|∫|√|π|equation|formula|calculate|compute|derivative|integral").unwrap();
    static ref LOGIC_PATTERN: Regex = Regex::new(r"(?i)(if |then |else |and |or |not |therefore |thus |because |implies |logical|reasoning|inference|deduction)").unwrap();
    static ref CREATIVE_PATTERN: Regex = Regex::new(r"(?i)(story|poem|novel|narrative|creative|write|compose|imagine|fictional|character|dialogue|scene)").unwrap();
    static ref DATA_PATTERN: Regex = Regex::new(r"(?i)(extract|parse|structure|json|csv|table|database|schema|data|analyze|statistics)").unwrap();
}

pub struct ComplexityScorer;

impl ComplexityScorer {
    /// Calculate a complexity score (1-10) based on prompt analysis
    pub fn calculate_complexity(prompt: &str) -> u8 {
        let mut score: u8 = 1;
        let prompt_len = prompt.len();

        // Length analysis
        if prompt_len > 1000 {
            score = score.saturating_add(3);
        } else if prompt_len > 500 {
            score = score.saturating_add(2);
        } else if prompt_len > 200 {
            score = score.saturating_add(1);
        }

        // Vocabulary complexity - count unique words
        let words: Vec<&str> = prompt.split_whitespace().collect();
        let unique_words = words.len();
        if unique_words > 200 {
            score = score.saturating_add(2);
        } else if unique_words > 100 {
            score = score.saturating_add(1);
        }

        // Code detection
        if CODE_PATTERN.find(prompt).is_some() {
            score = score.saturating_add(3);
        }

        // Mathematical content
        let math_matches = MATH_PATTERN.find_iter(prompt).count();
        if math_matches > 5 {
            score = score.saturating_add(3);
        } else if math_matches > 0 {
            score = score.saturating_add(1);
        }

        // Logical reasoning
        if LOGIC_PATTERN.find(prompt).is_some() {
            score = score.saturating_add(2);
        }

        // Creative content
        if CREATIVE_PATTERN.find(prompt).is_some() && CODE_PATTERN.find(prompt).is_none() && MATH_PATTERN.find(prompt).is_none() {
            // Creative tasks are less computationally complex
            score = score.saturating_sub(1);
        }

        // Data extraction
        if DATA_PATTERN.find(prompt).is_some() {
            score = score.saturating_add(2);
        }

        // Multiple tasks indicator
        if prompt.matches("and then").count() > 1 || prompt.matches(';').count() > 2 {
            score = score.saturating_add(2);
        }

        // Clamp score to 1-10 range
        score.clamp(1, 10)
    }

    /// Determine if complexity requires consensus from multiple models
    pub fn requires_consensus(score: u8, threshold: u8) -> bool {
        score >= threshold
    }

    /// Get a descriptive tier based on complexity score
    pub fn get_tier(score: u8) -> &'static str {
        match score {
            1..=3 => "instant",
            4..=7 => "analytical",
            8..=10 => "expert",
            _ => "unknown",
        }
    }

    /// Calculate resource budget based on complexity
    pub fn calculate_resource_budget(score: u8) -> ResourceBudget {
        match score {
            1..=3 => ResourceBudget {
                max_vram_mb: 512,
                max_latency_ms: 100,
                preferred_model_size: "1B".to_string(),
                parallel_models: 1,
            },
            4..=7 => ResourceBudget {
                max_vram_mb: 2048,
                max_latency_ms: 500,
                preferred_model_size: "3B".to_string(),
                parallel_models: 2,
            },
            8..=10 => ResourceBudget {
                max_vram_mb: 4096,
                max_latency_ms: 2000,
                preferred_model_size: "7B+".to_string(),
                parallel_models: 3,
            },
            _ => ResourceBudget::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceBudget {
    pub max_vram_mb: usize,
    pub max_latency_ms: u64,
    pub preferred_model_size: String,
    pub parallel_models: usize,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        ResourceBudget {
            max_vram_mb: 1024,
            max_latency_ms: 500,
            preferred_model_size: "1B".to_string(),
            parallel_models: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_scoring() {
        let simple = "Hello, how are you?";
        assert_eq!(ComplexityScorer::calculate_complexity(simple), 1);

        let code_prompt = "Write a Rust function that implements fn quicksort";
        let code_score = ComplexityScorer::calculate_complexity(code_prompt);
        assert!(code_score >= 4);

        let math_prompt = "Calculate the derivative of x^2 + 3x + 5";
        let math_score = ComplexityScorer::calculate_complexity(math_prompt);
        assert!(math_score >= 2);
    }

    #[test]
    fn test_tier_classification() {
        assert_eq!(ComplexityScorer::get_tier(2), "instant");
        assert_eq!(ComplexityScorer::get_tier(5), "analytical");
        assert_eq!(ComplexityScorer::get_tier(9), "expert");
    }
}
