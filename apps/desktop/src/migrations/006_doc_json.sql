-- Draffity schema v6 â€” TipTap JSON alongside HTML for documents.
--
-- The editor currently round-trips through HTML. That's lossy for some
-- TipTap node attributes and brittle for future analyses (diff per node,
-- per-paragraph stats, AI ops). We migrate to dual-storage: keep `content`
-- as a render-friendly HTML cache, add `content_json` as the canonical
-- ProseMirror state.
--
-- Aditiva: la columna es nullable. Documentos pre-existentes se quedan con
-- content_json = NULL hasta que se editen por primera vez; el editor
-- prefiere JSON cuando estÃ¡, cae a HTML si no.

ALTER TABLE documents ADD COLUMN content_json TEXT;
