-- Custom metadata fields (I-08/I-09): user-defined per-project fields attached
-- to documents. `custom_fields` holds the field definitions (name + kind +
-- optional select options); `document_custom_values` is the per-document value
-- store, keyed by (document, field). Distinct from template `metadataFields`
-- (project-level, defined by templates) — these are document-level and edited
-- by the writer at runtime. Free-tier. Additive migration.

CREATE TABLE IF NOT EXISTS custom_fields (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    kind         TEXT NOT NULL,         -- 'text' | 'number' | 'date' | 'select'
    options_json TEXT,                  -- select only: JSON array of strings
    position     INTEGER NOT NULL,
    created_at   INTEGER NOT NULL,
    UNIQUE (project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_custom_fields_project ON custom_fields(project_id);

CREATE TABLE IF NOT EXISTS document_custom_values (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    field_id    TEXT NOT NULL REFERENCES custom_fields(id) ON DELETE CASCADE,
    value       TEXT NOT NULL,
    PRIMARY KEY (document_id, field_id)
);

CREATE INDEX IF NOT EXISTS idx_doc_custom_values_field ON document_custom_values(field_id);
