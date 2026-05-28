-- Citation entries imported from BibTeX. One row per (project, key).
-- `fields_json` stores the rest of the entry as a flat JSON map so we don't
-- have to pick a fixed schema across BibTeX entry types (article, book,
-- inproceedings…). Type is preserved as a separate column for filtering.

CREATE TABLE IF NOT EXISTS citations (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key          TEXT NOT NULL,
    entry_type   TEXT NOT NULL,
    fields_json  TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    UNIQUE(project_id, key)
);

CREATE INDEX IF NOT EXISTS idx_citations_project ON citations(project_id);
