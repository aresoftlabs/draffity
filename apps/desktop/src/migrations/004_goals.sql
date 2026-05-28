-- Draffity schema v4 — word-count goals per project and per document.
--
-- Both columns are nullable. NULL = no goal set; the UI hides the progress
-- widget. Storing as INTEGER (number of words) so the UI computes the
-- percentage against the live word count without needing a separate field.

ALTER TABLE projects ADD COLUMN goal_words INTEGER;
ALTER TABLE documents ADD COLUMN goal_words INTEGER;
