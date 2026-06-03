-- Draffity schema v1
-- Free MVP. Feature tables (ai_cache, sync_state, codex_entries, voice_takes)
-- are reserved for future migrations in the 100_* range.

CREATE TABLE IF NOT EXISTS meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT OR IGNORE INTO meta(key, value) VALUES
  ('schema_version', '1'),
  ('tier', 'free');

CREATE TABLE IF NOT EXISTS projects (
  id           TEXT PRIMARY KEY,
  title        TEXT NOT NULL,
  template_id  TEXT NOT NULL,
  status       TEXT NOT NULL CHECK(status IN ('active','archived')),
  metadata     TEXT,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

-- Invariant: at most one project with status='active'.
CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_one_active
  ON projects(status) WHERE status='active';

CREATE INDEX IF NOT EXISTS idx_projects_updated_at ON projects(updated_at DESC);

CREATE TABLE IF NOT EXISTS documents (
  id          TEXT PRIMARY KEY,
  project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id   TEXT REFERENCES documents(id) ON DELETE CASCADE,
  title       TEXT NOT NULL,
  doc_type    TEXT NOT NULL,
  content     TEXT,
  position    INTEGER NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id);
CREATE INDEX IF NOT EXISTS idx_documents_parent ON documents(parent_id);
CREATE INDEX IF NOT EXISTS idx_documents_position ON documents(project_id, parent_id, position);

CREATE TABLE IF NOT EXISTS snapshots (
  id          TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  content     TEXT NOT NULL,
  label       TEXT,
  created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_snapshots_document ON snapshots(document_id);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
