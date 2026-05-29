//! A persisted AI validation report (G-02). The `results_json` holds the
//! serialized findings (`Vec<Finding>` from `services::ai_validators`); the
//! domain layer stays storage-shaped and doesn't depend on the service types.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiValidation {
    pub id: String,
    pub document_id: String,
    /// `character` | `voice` | `repetition` | `plot` | `style`.
    pub validator_name: String,
    /// JSON array of findings (rendered by the UI report view).
    pub results_json: String,
    /// Human summary, e.g. "1 crítico · 3 advertencias".
    pub severity_summary: String,
    pub created_at: i64,
}
