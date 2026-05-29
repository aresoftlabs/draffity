//! Media registry CRUD. The bytes themselves live on the filesystem under
//! `<app_data>/media/<project>/<hash>.<ext>` — this table just remembers
//! the mapping. Insert is unique on `(project_id, sha256)` so the second
//! paste of the same image returns the existing row instead of writing a
//! duplicate file.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, MediaAsset};
use crate::error::AppResult;

/// Look up the existing asset for `(project_id, sha256)`, if any. The
/// `LocalMediaService` calls this before writing bytes to dedupe.
pub(super) fn find_by_hash(
    conn: &Connection,
    project_id: &str,
    sha256: &str,
) -> AppResult<Option<MediaAsset>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, path_relative, mime, sha256, bytes, created_at, duration_ms, transcribed_text, is_voice_note
         FROM media WHERE project_id = ?1 AND sha256 = ?2",
    )?;
    let row = stmt
        .query_row(params![project_id, sha256], row_to_media)
        .optional()?;
    Ok(row)
}

/// Insert a fresh media row. Caller has already written the file and
/// computed `sha256` + `path_relative`. The unique index on
/// `(project_id, sha256)` is the safety net for races.
pub(super) fn insert(
    conn: &Connection,
    project_id: &str,
    path_relative: &str,
    mime: &str,
    sha256: &str,
    bytes: i64,
) -> AppResult<MediaAsset> {
    let id = new_id();
    let now = now_ms();
    conn.execute(
        "INSERT INTO media(id, project_id, path_relative, mime, sha256, bytes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_id, path_relative, mime, sha256, bytes, now],
    )?;
    Ok(MediaAsset {
        id,
        project_id: project_id.into(),
        path_relative: path_relative.into(),
        mime: mime.into(),
        sha256: sha256.into(),
        bytes,
        created_at: now,
        duration_ms: None,
        transcribed_text: None,
        is_voice_note: false,
    })
}

/// Flag a stored asset as a voice note (H), setting its duration + optional
/// transcription. Returns the refreshed row.
pub(super) fn set_voice_note(
    conn: &Connection,
    id: &str,
    duration_ms: Option<i64>,
    transcribed_text: Option<&str>,
) -> AppResult<MediaAsset> {
    conn.execute(
        "UPDATE media SET is_voice_note = 1, duration_ms = ?2, transcribed_text = ?3 WHERE id = ?1",
        params![id, duration_ms, transcribed_text],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("media {id}")))
}

/// List a project's voice notes, newest first.
pub(super) fn list_voice_notes(conn: &Connection, project_id: &str) -> AppResult<Vec<MediaAsset>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, path_relative, mime, sha256, bytes, created_at, duration_ms, transcribed_text, is_voice_note
         FROM media WHERE project_id = ?1 AND is_voice_note = 1 ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![project_id], row_to_media)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn get(conn: &Connection, id: &str) -> AppResult<Option<MediaAsset>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, path_relative, mime, sha256, bytes, created_at, duration_ms, transcribed_text, is_voice_note
         FROM media WHERE id = ?1",
    )?;
    Ok(stmt.query_row(params![id], row_to_media).optional()?)
}

pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<MediaAsset>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, path_relative, mime, sha256, bytes, created_at, duration_ms, transcribed_text, is_voice_note
         FROM media WHERE project_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt
        .query_map(params![project_id], row_to_media)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<Option<MediaAsset>> {
    let existing = get(conn, id)?;
    if existing.is_some() {
        conn.execute("DELETE FROM media WHERE id = ?1", params![id])?;
    }
    Ok(existing)
}

fn row_to_media(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaAsset> {
    Ok(MediaAsset {
        id: row.get(0)?,
        project_id: row.get(1)?,
        path_relative: row.get(2)?,
        mime: row.get(3)?,
        sha256: row.get(4)?,
        bytes: row.get(5)?,
        created_at: row.get(6)?,
        duration_ms: row.get(7)?,
        transcribed_text: row.get(8)?,
        is_voice_note: row.get::<_, i64>(9)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;

    #[test]
    fn insert_and_find_round_trip() {
        let s = fresh();
        let p = seed_project(&s, "P");
        // Use the public trait so we don't have to re-import the submodule.
        let asset = s
            .insert_media_row(&p.id, "media/p/h.jpg", "image/jpeg", "deadbeef", 1234)
            .unwrap();
        assert_eq!(asset.bytes, 1234);
        let found = s
            .find_media_by_hash(&p.id, "deadbeef")
            .unwrap()
            .expect("expected dedupe lookup to hit");
        assert_eq!(found.id, asset.id);
    }

    #[test]
    fn find_returns_none_for_unknown_hash() {
        let s = fresh();
        let p = seed_project(&s, "P");
        assert!(s.find_media_by_hash(&p.id, "ffff").unwrap().is_none());
    }

    #[test]
    fn list_returns_assets_in_creation_order() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let a = s
            .insert_media_row(&p.id, "media/p/a.png", "image/png", "h1", 1)
            .unwrap();
        // Slight sleep to ensure created_at is strictly greater.
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = s
            .insert_media_row(&p.id, "media/p/b.png", "image/png", "h2", 2)
            .unwrap();
        let list = s.list_media(&p.id).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, a.id);
        assert_eq!(list[1].id, b.id);
    }

    #[test]
    fn delete_removes_row_and_returns_asset() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let asset = s
            .insert_media_row(&p.id, "media/p/x.png", "image/png", "x", 10)
            .unwrap();
        let removed = s.delete_media_row(&asset.id).unwrap();
        assert_eq!(removed.unwrap().id, asset.id);
        assert!(s.get_media(&asset.id).unwrap().is_none());
    }

    #[test]
    fn delete_missing_id_is_noop() {
        let s = fresh();
        assert!(s.delete_media_row("ghost").unwrap().is_none());
    }

    #[test]
    fn unique_on_project_and_hash_prevents_duplicates() {
        let s = fresh();
        let p = seed_project(&s, "P");
        s.insert_media_row(&p.id, "media/p/a.png", "image/png", "h", 1)
            .unwrap();
        // Second insert with the same (project_id, sha256) must fail at
        // the SQL layer — the caller is expected to dedupe via
        // `find_media_by_hash` first.
        let err = s.insert_media_row(&p.id, "media/p/dup.png", "image/png", "h", 2);
        assert!(err.is_err());
    }

    #[test]
    fn voice_note_round_trip_and_listing() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let a = s
            .insert_media_row(&p.id, "media/p/v.wav", "audio/wav", "vh", 999)
            .unwrap();
        // Plain media isn't a voice note.
        assert!(!a.is_voice_note);
        assert!(s.list_voice_notes(&p.id).unwrap().is_empty());

        let updated = s
            .set_media_voice_note(&a.id, Some(4200), Some("hola que tal"))
            .unwrap();
        assert!(updated.is_voice_note);
        assert_eq!(updated.duration_ms, Some(4200));
        assert_eq!(updated.transcribed_text.as_deref(), Some("hola que tal"));

        let notes = s.list_voice_notes(&p.id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, a.id);
    }

    #[test]
    fn deleting_project_cascades_media_rows() {
        let s = fresh();
        let p = seed_project(&s, "P");
        s.insert_media_row(&p.id, "media/p/x.png", "image/png", "h", 1)
            .unwrap();
        s.delete_project(&p.id).unwrap();
        assert!(s.list_media(&p.id).unwrap().is_empty());
    }
}
