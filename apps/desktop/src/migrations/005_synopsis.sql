-- Draffity schema v5 — synopsis as a first-class field.
--
-- Until now, the `synopsis` from a template lived inside the seeded document's
-- `content` as a `<p>` block. That coupled the synopsis (a structural,
-- metadata-flavoured field surfaced in cards/outliner) with the editable
-- body. The new `synopsis` column separates them so Corkboard and Outliner
-- views can show synopses independently of content.
--
-- Nullable: existing documents stay as-is (synopsis = NULL); new ones can
-- opt in. Future content-rewrites can choose to populate this from the
-- first paragraph if desired — that decision is deferred.

ALTER TABLE documents ADD COLUMN synopsis TEXT;
