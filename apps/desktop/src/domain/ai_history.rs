//! AI generation history entry (F-12). A lightweight, append-only record of
//! an *accepted* AI generation — for transparency and reuse, not an audit of
//! every request.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiHistoryEntry {
    pub id: String,
    pub project_id: String,
    pub doc_id: Option<String>,
    /// The action that produced it: `continue` / `expand` / `rewrite` /
    /// `describe` (rewrite sub-mode is folded into the string, e.g.
    /// `rewrite:vivid`).
    pub action: String,
    pub model: Option<String>,
    pub response: String,
    pub created_at: i64,
}
