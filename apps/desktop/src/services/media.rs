//! Media service: writes blob bytes to disk under
//! `<app_data>/media/<project>/<sha256>.<ext>` and registers them via
//! `StorageService`. Dedupes by `(project_id, sha256)` so pasting the
//! same image twice references a single file on disk.
//!
//! Premium-ready behind `MediaService` trait + `NoOpMedia`. A future
//! `CloudMediaService` can implement the same surface against an S3
//! backend without touching commands or the editor.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::domain::{extension_for_mime, MediaAsset};
use crate::error::{AppError, AppResult};
use crate::services::storage::StorageService;

pub trait MediaService: Send + Sync {
    /// Persist `bytes` for `project_id`. Returns the registry row (existing
    /// when the hash already lives on disk, fresh otherwise).
    fn store(&self, project_id: &str, mime: &str, bytes: &[u8]) -> AppResult<MediaAsset>;
    /// Read the bytes of a previously stored asset. Errors if the registry
    /// row was deleted or the disk file is gone.
    fn read(&self, asset_id: &str) -> AppResult<Vec<u8>>;
    /// Delete the registry row + the disk file. Idempotent: missing rows
    /// return `Ok(())`.
    fn delete(&self, asset_id: &str) -> AppResult<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpMedia;

impl MediaService for NoOpMedia {
    fn store(&self, _project_id: &str, _mime: &str, _bytes: &[u8]) -> AppResult<MediaAsset> {
        Err(AppError::Unsupported("media storage disabled".into()))
    }
    fn read(&self, _asset_id: &str) -> AppResult<Vec<u8>> {
        Err(AppError::Unsupported("media storage disabled".into()))
    }
    fn delete(&self, _asset_id: &str) -> AppResult<()> {
        Ok(())
    }
}

pub struct LocalMediaService {
    storage: Arc<dyn StorageService>,
    root: PathBuf,
}

impl LocalMediaService {
    pub fn new(storage: Arc<dyn StorageService>, app_data_dir: &Path) -> Self {
        let root = app_data_dir.join("media");
        Self { storage, root }
    }

    fn absolute_path(&self, relative: &str) -> PathBuf {
        // `path_relative` is stored as `media/<project>/<hash>.<ext>` — the
        // `<app_data>` part is what we add back here. Strip the leading
        // `media/` since the root already includes it.
        let stripped = relative.strip_prefix("media/").unwrap_or(relative);
        self.root.join(stripped)
    }
}

impl MediaService for LocalMediaService {
    fn store(&self, project_id: &str, mime: &str, bytes: &[u8]) -> AppResult<MediaAsset> {
        let sha256 = hex_digest(bytes);

        // Dedupe: if the same project already has this sha256, return the
        // existing row without rewriting the file.
        if let Some(existing) = self.storage.find_media_by_hash(project_id, &sha256)? {
            return Ok(existing);
        }

        let ext = extension_for_mime(mime);
        let project_dir = self.root.join(project_id);
        std::fs::create_dir_all(&project_dir)?;
        let file_name = format!("{sha256}.{ext}");
        let abs_path = project_dir.join(&file_name);
        std::fs::write(&abs_path, bytes)?;

        let path_relative = format!("media/{project_id}/{file_name}");
        self.storage.insert_media_row(
            project_id,
            &path_relative,
            mime,
            &sha256,
            bytes.len() as i64,
        )
    }

    fn read(&self, asset_id: &str) -> AppResult<Vec<u8>> {
        let asset = self
            .storage
            .get_media(asset_id)?
            .ok_or_else(|| AppError::NotFound(format!("media {asset_id}")))?;
        let path = self.absolute_path(&asset.path_relative);
        Ok(std::fs::read(&path)?)
    }

    fn delete(&self, asset_id: &str) -> AppResult<()> {
        let Some(asset) = self.storage.delete_media_row(asset_id)? else {
            return Ok(());
        };
        let path = self.absolute_path(&asset.path_relative);
        if let Err(e) = std::fs::remove_file(&path) {
            // Best-effort: a missing file means the row was already orphaned.
            // We only log loud errors.
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(file = %path.display(), error = %e, "failed to delete media file");
            }
        }
        Ok(())
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::storage::LocalStorageService;

    fn tempdir(prefix: &str) -> std::path::PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("draffity-{prefix}-{n:x}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn setup() -> (Arc<LocalStorageService>, LocalMediaService, String, PathBuf) {
        let dir = tempdir("media");
        let path = dir.join("draffity.db");
        let storage = LocalStorageService::open(&path).unwrap();
        storage.migrate().unwrap();
        let storage_arc = Arc::new(storage);
        let project = storage_arc
            .create_project(crate::domain::ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let svc = LocalMediaService::new(storage_arc.clone(), &dir);
        (storage_arc, svc, project.id, dir)
    }

    #[test]
    fn store_writes_file_and_returns_asset() {
        let (_storage, svc, project_id, dir) = setup();
        let bytes = b"hello world";
        let asset = svc.store(&project_id, "image/png", bytes).unwrap();
        assert_eq!(asset.mime, "image/png");
        assert_eq!(asset.bytes, bytes.len() as i64);
        assert!(asset.path_relative.starts_with("media/"));
        // File exists on disk.
        let abs = dir.join("media").join(&project_id);
        let entries: Vec<_> = std::fs::read_dir(abs).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn store_dedupes_by_hash() {
        let (_storage, svc, project_id, _dir) = setup();
        let bytes = b"same content";
        let a = svc.store(&project_id, "image/png", bytes).unwrap();
        let b = svc.store(&project_id, "image/png", bytes).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.path_relative, b.path_relative);
    }

    #[test]
    fn read_returns_bytes_round_trip() {
        let (_storage, svc, project_id, _dir) = setup();
        let bytes = b"hola mundo".to_vec();
        let asset = svc.store(&project_id, "image/png", &bytes).unwrap();
        let read_back = svc.read(&asset.id).unwrap();
        assert_eq!(read_back, bytes);
    }

    #[test]
    fn read_unknown_id_is_not_found() {
        let (_storage, svc, _project_id, _dir) = setup();
        let err = svc.read("ghost").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn delete_removes_row_and_file() {
        let (storage, svc, project_id, _dir) = setup();
        let asset = svc.store(&project_id, "image/png", b"x").unwrap();
        let abs = svc.absolute_path(&asset.path_relative);
        assert!(abs.exists());
        svc.delete(&asset.id).unwrap();
        assert!(storage.get_media(&asset.id).unwrap().is_none());
        assert!(!abs.exists());
    }

    #[test]
    fn delete_missing_id_is_noop() {
        let (_storage, svc, _project_id, _dir) = setup();
        svc.delete("ghost").unwrap();
    }

    #[test]
    fn noop_rejects_writes_returns_ok_on_delete() {
        let svc = NoOpMedia;
        assert!(svc.store("p", "image/png", b"x").is_err());
        assert!(svc.read("id").is_err());
        assert!(svc.delete("id").is_ok());
    }
}
