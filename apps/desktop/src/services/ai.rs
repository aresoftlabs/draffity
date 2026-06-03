//! AI service abstraction.
//!
//! The default impl is `NoOpAI` (reports unavailable). The BYOK impl
//! `OpenRouterAIService` implements this trait and is wired in when an API key
//! is present — no core change needed.
//!
//! Streaming is modelled with a **callback sink** (`&mut dyn FnMut(&str)`)
//! rather than a returned `Stream`. This keeps the trait object-safe
//! (`Arc<dyn AIService>`) and avoids pulling an async runtime into a
//! currently-sync codebase: the BYOK impl does blocking SSE reads and
//! forwards each delta to the sink, which the command layer relays to the UI
//! via Tauri events. See `backlog-v4.md` E-03 / F-01.

use crate::error::AppResult;

/// Role of a message in the conversation sent to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// A model request. `model` selects the OpenRouter model id; when `None` the
/// impl falls back to a configured default. Sampling params are optional so
/// callers only set what they care about.
#[derive(Debug, Clone, Default)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl CompletionRequest {
    /// Convenience constructor for a single user prompt with no system message.
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        Self {
            messages: vec![ChatMessage::user(prompt)],
            ..Default::default()
        }
    }
}

/// Token accounting reported by the provider, used by the cost meter (F-13).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionResponse {
    pub text: String,
    pub usage: Option<TokenUsage>,
}

pub trait AIService: Send + Sync {
    fn available(&self) -> bool;

    /// Non-streaming completion. Default errors out (no key / NoOp impl).
    fn complete(&self, _req: CompletionRequest) -> AppResult<CompletionResponse> {
        Err(crate::error::AppError::Unsupported(
            "las funciones de IA no están disponibles".into(),
        ))
    }

    /// Streaming completion: `sink` receives each text delta as it arrives;
    /// the assembled response (with usage) is returned at the end.
    ///
    /// The default delegates to [`complete`] and feeds the whole text to the
    /// sink in one shot, so an alternative impl can implement either method.
    fn stream_complete(
        &self,
        req: CompletionRequest,
        sink: &mut dyn FnMut(&str),
    ) -> AppResult<CompletionResponse> {
        let resp = self.complete(req)?;
        sink(&resp.text);
        Ok(resp)
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
        assert!(ai.complete(CompletionRequest::from_prompt("x")).is_err());
    }

    #[test]
    fn noop_stream_complete_errors() {
        let ai = NoOpAI;
        let mut deltas = String::new();
        let res = ai.stream_complete(CompletionRequest::from_prompt("x"), &mut |d| {
            deltas.push_str(d)
        });
        assert!(res.is_err());
        assert!(deltas.is_empty(), "no deltas should be emitted on error");
    }

    #[test]
    fn from_prompt_builds_single_user_message() {
        let req = CompletionRequest::from_prompt("hello");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, Role::User);
        assert_eq!(req.messages[0].content, "hello");
        assert!(req.model.is_none());
    }
}
