-- Media blob registry. The actual bytes live on disk at
-- `<app_data>/media/<project_id>/<sha256>.<ext>`; this table tracks the
-- relationship + dedupes by (project_id, sha256) so pasting the same image
-- twice references a single file.
--
-- Cascade delete from projects keeps the row count consistent; the
-- filesystem files are cleaned up by the MediaService when a row is
-- removed (best-effort).

CREATE TABLE IF NOT EXISTS media (
    id            TEXT PRIMARY KEY,
    project_id    TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path_relative TEXT NOT NULL,
    mime          TEXT NOT NULL,
    sha256        TEXT NOT NULL,
    bytes         INTEGER NOT NULL,
    created_at    INTEGER NOT NULL,
    UNIQUE(project_id, sha256)
);

CREATE INDEX IF NOT EXISTS idx_media_project ON media(project_id);
