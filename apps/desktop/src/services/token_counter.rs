//! Token estimation (F-02). A **pre-send heuristic**, not an exact tokenizer.
//!
//! OpenRouter routes to many models with different tokenizers, so exact
//! counting client-side is impossible (and `tiktoken` would only match the
//! OpenAI family). The real usage comes back in the streamed response and
//! drives the cost meter (F-13); this estimate only needs to be roughly right
//! to warn the user before sending. The ~4-chars-per-token rule holds well
//! enough for Spanish/English prose.

use crate::services::ai::ChatMessage;

/// Approximate token count of a text fragment. Rounds up so non-empty input
/// never estimates to zero.
pub fn estimate_tokens(text: &str) -> u32 {
    let chars = text.chars().count() as u32;
    chars.div_ceil(4)
}

/// Approximate prompt-token count of a full chat request: content of every
/// message plus a small fixed per-message overhead (role + delimiters) and a
/// priming constant, mirroring how chat APIs bill message envelopes.
pub fn estimate_request_tokens(messages: &[ChatMessage]) -> u32 {
    const PER_MESSAGE_OVERHEAD: u32 = 4;
    const PRIMING: u32 = 2;
    messages
        .iter()
        .map(|m| estimate_tokens(&m.content) + PER_MESSAGE_OVERHEAD)
        .sum::<u32>()
        + PRIMING
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::ChatMessage;

    #[test]
    fn empty_text_is_zero_tokens() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn short_text_rounds_up_to_at_least_one() {
        assert_eq!(estimate_tokens("a"), 1);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }

    #[test]
    fn longer_text_estimates_more_than_shorter() {
        let short = estimate_tokens("hola mundo");
        let long = estimate_tokens(&"hola mundo ".repeat(50));
        assert!(long > short);
    }

    #[test]
    fn request_tokens_account_for_message_overhead() {
        let msgs = vec![
            ChatMessage::system("eres un editor"),
            ChatMessage::user("continúa esta escena"),
        ];
        let est = estimate_request_tokens(&msgs);
        // Strictly greater than the bare content sum, due to overhead/priming.
        let content_only =
            estimate_tokens("eres un editor") + estimate_tokens("continúa esta escena");
        assert!(est > content_only);
    }
}
