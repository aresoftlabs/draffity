-- AI validation reports (G-02). One row per (document, validator) run; the
-- findings are stored as JSON so re-opening a document shows the last report
-- without calling the model again. Additive migration.

CREATE TABLE IF NOT EXISTS ai_validations (
    id               TEXT PRIMARY KEY,
    document_id      TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    validator_name   TEXT NOT NULL,
    results_json     TEXT NOT NULL,
    severity_summary TEXT NOT NULL,
    created_at       INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_validations_doc
    ON ai_validations(document_id, created_at DESC);
