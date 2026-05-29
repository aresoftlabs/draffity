-- Project deadline (J-02): an optional target date for finishing the project,
-- stored as epoch milliseconds (UTC) like every other timestamp. Powers the
-- pacemaker widget (J-03: words/day needed = words remaining / days left).
-- Additive, non-destructive.

ALTER TABLE projects
  ADD COLUMN deadline INTEGER;
