-- Front/back matter (K-03): mark documents that compile before (front) or
-- after (back) the main manuscript body — title page, dedication, appendix,
-- acknowledgements. At export time these top-level docs are reordered to the
-- start/end. Additive, non-destructive (existing rows default to 0).

ALTER TABLE documents ADD COLUMN is_front_matter INTEGER NOT NULL DEFAULT 0;
ALTER TABLE documents ADD COLUMN is_back_matter INTEGER NOT NULL DEFAULT 0;
