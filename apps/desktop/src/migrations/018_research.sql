-- Research folder (I-10): documents flagged as research material. A research
-- document (and everything nested under it) is excluded from project word-count
-- stats and from export by default (opt-in via ExportConfig.include_research).
-- Additive, non-destructive: existing rows default to 0 (not research).

ALTER TABLE documents
  ADD COLUMN is_research INTEGER NOT NULL DEFAULT 0;
