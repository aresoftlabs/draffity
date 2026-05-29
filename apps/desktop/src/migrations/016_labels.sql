-- Labels (I-05/I-06): per-project colored labels, many-to-many with documents.
-- A document can carry several labels; a label belongs to exactly one project.
-- Colored chips surface in the binder / outliner / corkboard / inspector and
-- can be used as a binder filter. Free-tier feature. Additive migration.

CREATE TABLE IF NOT EXISTS labels (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL,          -- hex string, e.g. '#ef4444'
    created_at  INTEGER NOT NULL,
    UNIQUE (project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_labels_project ON labels(project_id);

CREATE TABLE IF NOT EXISTS document_labels (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    label_id    TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_document_labels_label ON document_labels(label_id);
