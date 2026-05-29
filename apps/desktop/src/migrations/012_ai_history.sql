-- AI generation history (F-12). Only *accepted* generations are persisted,
-- for transparency ("which parts of the manuscript came from AI") and reuse.
-- Rejected/cancelled generations never land here. Additive migration.

CREATE TABLE IF NOT EXISTS ai_history (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    doc_id      TEXT,
    action      TEXT NOT NULL,
    model       TEXT,
    response    TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_history_project
    ON ai_history(project_id, created_at DESC);
