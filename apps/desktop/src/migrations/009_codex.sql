-- Codex entries: worldbuilding artefacts (characters, places, objects, notes)
-- attached to a project. Cross-references in the editor target `id`, never
-- `name`, so renames don't break the manuscript.

CREATE TABLE IF NOT EXISTS codex_entries (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,            -- character | place | object | note
    name        TEXT NOT NULL,
    body        TEXT,                     -- free-form HTML/markdown body
    tags_json   TEXT NOT NULL DEFAULT '[]',
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_codex_project ON codex_entries(project_id);
CREATE INDEX IF NOT EXISTS idx_codex_project_kind ON codex_entries(project_id, kind);
-- Lookup by name within a project is used by the editor's "insert cross-ref"
-- picker; not unique because users might legitimately reuse a name across
-- kinds ("Mordor" the place vs. "Mordor" the song).
CREATE INDEX IF NOT EXISTS idx_codex_project_name ON codex_entries(project_id, name);
