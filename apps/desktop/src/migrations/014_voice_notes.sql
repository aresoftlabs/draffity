-- Voice notes (H-08). Reuse the `media` registry: a voice note is a stored
-- audio blob flagged `is_voice_note`, with its duration and an optional
-- transcription. Additive — existing media rows default to non-voice-note.

ALTER TABLE media ADD COLUMN duration_ms INTEGER;
ALTER TABLE media ADD COLUMN transcribed_text TEXT;
ALTER TABLE media ADD COLUMN is_voice_note INTEGER NOT NULL DEFAULT 0;
