-- Draffity schema v2 — full-text search.
--
-- FTS5 mirror of `documents` kept in sync by triggers. We intentionally use
-- a regular (non-external-content) FTS5 table so the index can be updated
-- with ordinary INSERT/DELETE. The duplication of `title` and `content` is
-- the price for trigger reliability across SQLite builds.
--
-- We index the raw HTML from `content` for now; a future iteration may
-- pre-strip tags before insertion for better relevance.

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
  title,
  content,
  tokenize='unicode61 remove_diacritics 2'
);

-- Backfill from existing rows (no-op on a fresh DB).
INSERT INTO documents_fts(rowid, title, content)
SELECT rowid, title, COALESCE(content, '') FROM documents;

CREATE TRIGGER IF NOT EXISTS documents_fts_ai AFTER INSERT ON documents BEGIN
  INSERT INTO documents_fts(rowid, title, content)
  VALUES (new.rowid, new.title, COALESCE(new.content, ''));
END;

CREATE TRIGGER IF NOT EXISTS documents_fts_ad AFTER DELETE ON documents BEGIN
  DELETE FROM documents_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS documents_fts_au AFTER UPDATE ON documents BEGIN
  DELETE FROM documents_fts WHERE rowid = old.rowid;
  INSERT INTO documents_fts(rowid, title, content)
  VALUES (new.rowid, new.title, COALESCE(new.content, ''));
END;
