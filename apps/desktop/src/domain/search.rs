//! Domain types for full-text search results.

use serde::{Deserialize, Serialize};

/// One match from a project-scoped FTS search. `excerpt` contains a snippet
/// of the matched content with HTML `<mark>` tags around the hit terms,
/// ready to be rendered as innerHTML in the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub document_id: String,
    pub project_id: String,
    pub title: String,
    pub excerpt: String,
}
