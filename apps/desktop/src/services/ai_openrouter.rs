//! BYOK OpenRouter implementation of `AIService`. Gated by the stored key at call time.
//!
//! The key lives in the OS keyring (E-01), never in SQLite. `available()` and
//! every call check only whether a non-empty key is present — no tier or
//! capability check.
//!
//! Streaming uses `reqwest::blocking` + manual SSE parsing, forwarding each
//! delta to the callback sink — fits the sync, object-safe `AIService`
//! contract (E-03) without an async runtime. Callers MUST invoke this off the
//! main thread (e.g. `spawn_blocking`); the blocking client panics if driven
//! from inside a Tokio runtime. The command layer (slice 2) owns that.

use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::ai::{AIService, CompletionRequest, CompletionResponse, Role, TokenUsage};
use crate::services::secrets::SecretStorage;

/// Keyring entry name for the BYOK OpenRouter key.
pub const OPENROUTER_KEY: &str = "openrouter_api_key";

// --- OpenRouter tunables (compile-time config; not user-facing) ---
const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// Default model when the request doesn't pin one. Cheap + capable; the UI
/// can override per request later (Épica F follow-ups).
const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
const MAX_RETRIES: u32 = 2;
/// Per-request HTTP timeout. Generous because completions can stream for a while.
const REQUEST_TIMEOUT_SECS: u64 = 120;
/// Base for the exponential retry backoff (`base * 2^attempt`).
const RETRY_BACKOFF_BASE_MS: u64 = 400;

pub struct ByokAIService {
    secrets: Arc<dyn SecretStorage>,
}

impl ByokAIService {
    pub fn new(secrets: Arc<dyn SecretStorage>) -> Self {
        Self { secrets }
    }

    fn gate(&self) -> AppResult<String> {
        self.secrets
            .get_secret(OPENROUTER_KEY)?
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| AppError::AiProvider("falta la API key de OpenRouter".into()))
    }

    fn send_with_retry(
        &self,
        api_key: &str,
        body: &ChatBody,
    ) -> AppResult<reqwest::blocking::Response> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| AppError::AiProvider(format!("cliente HTTP: {e}")))?;

        let mut attempt = 0u32;
        loop {
            let result = client
                .post(API_URL)
                .bearer_auth(api_key)
                .header("HTTP-Referer", "https://draffity.app")
                .header("X-Title", "Draffity")
                .json(body)
                .send();

            let retryable = match &result {
                Ok(r) => {
                    let s = r.status();
                    s == reqwest::StatusCode::TOO_MANY_REQUESTS || s.is_server_error()
                }
                Err(_) => true,
            };

            match result {
                Ok(r) if r.status().is_success() => return Ok(r),
                Ok(r) if retryable && attempt < MAX_RETRIES => {
                    tracing::warn!(status = %r.status(), attempt, "openrouter retryable status");
                }
                Ok(r) => {
                    let status = r.status();
                    let detail = r.text().unwrap_or_default();
                    return Err(AppError::AiProvider(format!(
                        "OpenRouter devolvió {status}: {}",
                        detail.chars().take(300).collect::<String>()
                    )));
                }
                Err(e) if attempt < MAX_RETRIES => {
                    tracing::warn!(error = %e, attempt, "openrouter request error, retrying");
                }
                Err(e) => return Err(AppError::AiProvider(format!("petición fallida: {e}"))),
            }

            attempt += 1;
            std::thread::sleep(Duration::from_millis(
                RETRY_BACKOFF_BASE_MS * 2u64.pow(attempt),
            ));
        }
    }
}

impl AIService for ByokAIService {
    fn available(&self) -> bool {
        self.secrets
            .get_secret(OPENROUTER_KEY)
            .ok()
            .flatten()
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    }

    fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        // Non-streaming = stream into a discarding sink.
        self.stream_complete(req, &mut |_| {})
    }

    fn stream_complete(
        &self,
        req: CompletionRequest,
        sink: &mut dyn FnMut(&str),
    ) -> AppResult<CompletionResponse> {
        let api_key = self.gate()?;
        let model = req
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let body = ChatBody {
            model,
            messages: req
                .messages
                .iter()
                .map(|m| WireMsg {
                    role: role_str(m.role),
                    content: m.content.clone(),
                })
                .collect(),
            stream: true,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream_options: StreamOptions {
                include_usage: true,
            },
        };
        let response = self.send_with_retry(&api_key, &body)?;
        parse_sse_stream(BufReader::new(response), sink)
    }
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

#[derive(Serialize)]
struct ChatBody {
    model: String,
    messages: Vec<WireMsg>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream_options: StreamOptions,
}

#[derive(Serialize)]
struct WireMsg {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: Delta,
}

#[derive(Deserialize, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct WireUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

/// Parse an OpenRouter SSE stream: lines of `data: {json}` ending in
/// `data: [DONE]`. Each chunk contributes a delta (forwarded to `sink`) and
/// the final chunk carries usage. Non-JSON lines (comments / keepalives) are
/// ignored. Pure over any `BufRead`, so it's unit-tested without the network.
pub(crate) fn parse_sse_stream<R: BufRead>(
    reader: R,
    sink: &mut dyn FnMut(&str),
) -> AppResult<CompletionResponse> {
    let mut text = String::new();
    let mut usage: Option<TokenUsage> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::AiProvider(format!("lectura del stream: {e}")))?;
        let Some(data) = line.trim().strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data == "[DONE]" {
            break;
        }
        let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) else {
            continue;
        };
        if let Some(content) = chunk
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.delta.content)
        {
            if !content.is_empty() {
                text.push_str(&content);
                sink(&content);
            }
        }
        if let Some(u) = chunk.usage {
            usage = Some(TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            });
        }
    }

    Ok(CompletionResponse { text, usage })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::secrets::InMemorySecretStorage;
    use std::io::Cursor;

    fn svc(key: Option<&str>) -> ByokAIService {
        let secrets = InMemorySecretStorage::new();
        if let Some(k) = key {
            secrets.set_secret(OPENROUTER_KEY, k).unwrap();
        }
        ByokAIService::new(Arc::new(secrets))
    }

    #[test]
    fn unavailable_without_key() {
        let s = svc(None);
        assert!(!s.available());
        assert!(s.complete(CompletionRequest::from_prompt("x")).is_err());
    }

    #[test]
    fn available_with_key() {
        let s = svc(Some("sk-abc"));
        assert!(s.available());
    }

    #[test]
    fn sse_parse_assembles_text_and_usage_and_streams_deltas() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hola\"}}]}\n",
            "\n",
            ": openrouter comment keepalive\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" mundo\"}}]}\n",
            "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":12,\"completion_tokens\":3}}\n",
            "data: [DONE]\n",
        );
        let mut streamed = String::new();
        let resp = parse_sse_stream(Cursor::new(body), &mut |d| streamed.push_str(d)).unwrap();
        assert_eq!(resp.text, "Hola mundo");
        assert_eq!(streamed, "Hola mundo");
        let usage = resp.usage.expect("usage parsed");
        assert_eq!(usage.prompt_tokens, 12);
        assert_eq!(usage.completion_tokens, 3);
    }

    #[test]
    fn sse_parse_ignores_malformed_lines() {
        let body = concat!(
            "garbage line without data prefix\n",
            "data: not-json\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n",
            "data: [DONE]\n",
        );
        let resp = parse_sse_stream(Cursor::new(body), &mut |_| {}).unwrap();
        assert_eq!(resp.text, "ok");
    }
}
