-- Draffity schema v3 â€” document status and tags.
--
-- `status` lets writers flag chapters/scenes through the writing pipeline
-- (Draft â†’ Revised â†’ Final, plus Trashed as a soft delete). Defaults to
-- 'draft' so existing rows keep working without backfill.
--
-- `document_tags` is a small adjacency table â€” many tags per document,
-- many documents per tag â€” with a covering index for fast tag filtering.

ALTER TABLE documents
  ADD COLUMN status TEXT NOT NULL DEFAULT 'draft'
  CHECK(status IN ('draft','revised','final','trashed'));

CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);

CREATE TABLE IF NOT EXISTS document_tags (
  document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  tag         TEXT NOT NULL,
  PRIMARY KEY (document_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_document_tags_tag ON document_tags(tag);
