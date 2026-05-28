-- Per-day writing activity counter. Tracks positive word-count deltas
-- across `update_document` calls so the Settings UI can render a 30-day
-- sparkline of "words written per day".
--
-- `words` is a sum of positive deltas only (we don't subtract for
-- deletions — the chart measures forward progress, not net change).
-- `sessions` counts distinct save events on that date and is a coarse
-- proxy for "did the user write today" independent of whether they
-- added or removed text.

CREATE TABLE IF NOT EXISTS daily_writing (
    date       TEXT PRIMARY KEY,
    words      INTEGER NOT NULL DEFAULT 0,
    sessions   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);
