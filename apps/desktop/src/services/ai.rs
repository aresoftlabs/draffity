//! AI service abstraction. **Premium-ready stub.**
//!
//! In the free MVP this is a `NoOpAI` that reports unavailable. Premium adds
//! `OpenRouterAIService` (BYOK) by implementing this trait — no core change.

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub text: String,
}

pub trait AIService: Send + Sync {
    fn available(&self) -> bool;
    fn complete(&self, _req: CompletionRequest) -> AppResult<CompletionResponse> {
        Err(crate::error::AppError::Unsupported(
            "AI features not available in free tier".into(),
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpAI;

impl AIService for NoOpAI {
    fn available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_unavailable() {
        let ai = NoOpAI;
        assert!(!ai.available());
        assert!(ai
            .complete(CompletionRequest {
                prompt: "x".into(),
                max_tokens: None,
            })
            .is_err());
    }
}
