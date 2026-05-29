-- Collections (I-01..I-03): saved groupings of documents, Scrivener-style.
-- `manual` collections hold an explicit ordered list (collection_documents);
-- `smart` collections store a serialized query (query_json) resolved live
-- against the project's documents. Additive migration.

CREATE TABLE IF NOT EXISTS collections (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL,          -- 'manual' | 'smart'
    query_json  TEXT,                   -- smart only
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_collections_project ON collections(project_id);

CREATE TABLE IF NOT EXISTS collection_documents (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    document_id   TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    position      INTEGER NOT NULL,
    PRIMARY KEY (collection_id, document_id)
);
