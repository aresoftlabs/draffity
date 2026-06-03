//! Centralised resource path management. All file-system paths that the
//! application cares about are computed from a single `DraffityHome` root,
//! defaulting to `$HOME/.draffity/` (or `%USERPROFILE%\.draffity\` on
//! Windows). A `config.json` at the root may override the path.
//!
//! A one-time `run_migration()` copies data from a legacy Tauri
//! `app_data_dir` so existing users keep their files.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Config persisted at `<root>/config.json`
#[derive(Deserialize, Serialize)]
struct DraffityConfig {
    root: Option<String>, // None = use default ~/.draffity/
}

/// Central access point for all app resource directories.
///
/// ```
/// let home = draffity_desktop_lib::services::DraffityHome::with_root(
///     std::path::PathBuf::from("/tmp/test-root"),
/// );
/// assert!(home.root().to_string_lossy().contains("test-root"));
/// assert!(home.db_path().to_string_lossy().ends_with("draffity.db"));
/// ```
pub struct DraffityHome {
    root: PathBuf,
}

impl DraffityHome {
    /// Build from the default path (`~/.draffity` or `%USERPROFILE%\.draffity`),
    /// probing `config.json` for an override.
    pub fn new() -> Self {
        let default = dirs::home_dir()
            .map(|h| h.join(".draffity"))
            .unwrap_or_else(|| PathBuf::from(".draffity"));
        let mut home = Self { root: default };
        if let Ok(config) = Self::load_config(&home.root) {
            if let Some(custom) = config.root.filter(|p| !p.is_empty()) {
                home.root = PathBuf::from(custom);
            }
        }
        home
    }
}

impl Default for DraffityHome {
    fn default() -> Self {
        Self::new()
    }
}

impl DraffityHome {
    /// For tests — build with an explicit root.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// The root directory path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // --- Subdirectory accessors ---

    pub fn voice_dir(&self) -> PathBuf {
        self.root.join("voice")
    }

    pub fn bin_dir(&self) -> PathBuf {
        let name = if cfg!(windows) {
            "whisper.exe"
        } else {
            "whisper"
        };
        self.voice_dir().join("bin").join(name)
    }

    pub fn piper_bin_path(&self) -> PathBuf {
        let name = if cfg!(windows) { "piper.exe" } else { "piper" };
        self.voice_dir().join("bin").join(name)
    }

    pub fn models_dir(&self) -> PathBuf {
        self.voice_dir().join("models")
    }

    pub fn model_path(&self, filename: &str) -> PathBuf {
        self.models_dir().join(filename)
    }

    pub fn voices_dir(&self) -> PathBuf {
        self.voice_dir().join("voices")
    }

    pub fn voice_file_path(&self, filename: &str) -> PathBuf {
        self.voices_dir().join(filename)
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.voice_dir().join("tmp")
    }

    pub fn media_dir(&self) -> PathBuf {
        self.root.join("media")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.root.join("backups")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn crash_reports_dir(&self) -> PathBuf {
        self.root.join("crash-reports")
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates").join("user")
    }

    pub fn db_path(&self) -> PathBuf {
        self.root.join("draffity.db")
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Create all subdirectories. Called once at startup.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for d in &[
            &self.voice_dir(),
            &self.models_dir(),
            &self.voices_dir(),
            &self.tmp_dir(),
            &self.media_dir(),
            &self.backups_dir(),
            &self.logs_dir(),
            &self.crash_reports_dir(),
            &self.root.join("templates"),
        ] {
            std::fs::create_dir_all(d)?;
        }
        Ok(())
    }

    /// Persist a custom root. Creates parent dirs and writes `config.json`.
    pub fn set_root(&mut self, path: PathBuf) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config = DraffityConfig {
            root: Some(path.to_string_lossy().into_owned()),
        };
        std::fs::write(self.config_path(), serde_json::to_string_pretty(&config)?)?;
        self.root = path;
        Ok(())
    }

    /// One-time migration from an old `app_data_dir`. Copies voice/, media/,
    /// backups/, crash-reports/, logs/, templates/, and `draffity.db` into
    /// this home, then writes a `.migrated-from` marker file.
    ///
    /// Returns `true` when data was actually copied, `false` when migration
    /// was skipped (already migrated, or target already has a db).
    pub fn run_migration(&self, old_app_data: &Path) -> AppResult<bool> {
        let marker = self.root.join(".migrated-from");
        if marker.exists() || self.db_path().exists() {
            return Ok(false); // already done or target already has data
        }

        let pairs = [
            (old_app_data.join("voice"), self.voice_dir()),
            (old_app_data.join("media"), self.media_dir()),
            (old_app_data.join("backups"), self.backups_dir()),
            (old_app_data.join("crash-reports"), self.crash_reports_dir()),
            (old_app_data.join("logs"), self.logs_dir()),
            (old_app_data.join("templates"), self.root.join("templates")),
            (old_app_data.join("draffity.db"), self.db_path()),
        ];

        self.ensure_dirs()?;
        for (src, dst) in &pairs {
            if src.is_dir() {
                copy_dir_all(src, dst)?;
            } else if src.is_file() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(src, dst)?;
            }
        }
        std::fs::write(&marker, old_app_data.to_string_lossy().as_bytes())?;
        Ok(true)
    }

    fn load_config(root: &Path) -> AppResult<DraffityConfig> {
        let path = root.join("config.json");
        if !path.exists() {
            return Ok(DraffityConfig { root: None });
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Path resolution — pure path ops, no filesystem needed
    // -----------------------------------------------------------------------

    #[test]
    fn with_root_sets_root_and_paths_are_relative() {
        let home = DraffityHome::with_root(PathBuf::from("/tmp/test-draffity"));
        assert_eq!(home.root(), Path::new("/tmp/test-draffity"));
        assert_eq!(home.voice_dir(), Path::new("/tmp/test-draffity/voice"));
        assert_eq!(
            home.models_dir(),
            Path::new("/tmp/test-draffity/voice/models")
        );
        assert_eq!(
            home.voices_dir(),
            Path::new("/tmp/test-draffity/voice/voices")
        );
        assert_eq!(home.tmp_dir(), Path::new("/tmp/test-draffity/voice/tmp"));
        assert_eq!(home.media_dir(), Path::new("/tmp/test-draffity/media"));
        assert_eq!(home.backups_dir(), Path::new("/tmp/test-draffity/backups"));
        assert_eq!(home.logs_dir(), Path::new("/tmp/test-draffity/logs"));
        assert_eq!(
            home.crash_reports_dir(),
            Path::new("/tmp/test-draffity/crash-reports")
        );
        assert_eq!(
            home.templates_dir(),
            Path::new("/tmp/test-draffity/templates/user")
        );
        assert_eq!(home.db_path(), Path::new("/tmp/test-draffity/draffity.db"));
        assert_eq!(
            home.config_path(),
            Path::new("/tmp/test-draffity/config.json")
        );
    }

    #[test]
    fn model_path_resolves_under_models_dir() {
        let home = DraffityHome::with_root(PathBuf::from("/r"));
        assert_eq!(
            home.model_path("ggml-base.bin"),
            Path::new("/r/voice/models/ggml-base.bin")
        );
    }

    #[test]
    fn voice_file_path_resolves_under_voices_dir() {
        let home = DraffityHome::with_root(PathBuf::from("/r"));
        assert_eq!(
            home.voice_file_path("es_ES-carlfm.onnx"),
            Path::new("/r/voice/voices/es_ES-carlfm.onnx")
        );
    }

    #[test]
    fn bin_dir_points_to_platform_specific_name() {
        let home = DraffityHome::with_root(PathBuf::from("/r"));
        let bin = home.bin_dir();
        let piper = home.piper_bin_path();
        if cfg!(windows) {
            assert!(bin.to_string_lossy().ends_with("whisper.exe"));
            assert!(piper.to_string_lossy().ends_with("piper.exe"));
        } else {
            assert!(bin.to_string_lossy().ends_with("whisper"));
            assert!(piper.to_string_lossy().ends_with("piper"));
        }
    }

    // -----------------------------------------------------------------------
    // ensure_dirs — creates all subdirs
    // -----------------------------------------------------------------------

    #[test]
    fn ensure_dirs_creates_all_subdirectories() {
        let tmp = tempfile::tempdir().unwrap();
        let home = DraffityHome::with_root(tmp.path().to_path_buf());
        home.ensure_dirs().unwrap();
        assert!(home.voice_dir().is_dir());
        assert!(home.models_dir().is_dir());
        assert!(home.voices_dir().is_dir());
        assert!(home.tmp_dir().is_dir());
        assert!(home.media_dir().is_dir());
        assert!(home.backups_dir().is_dir());
        assert!(home.logs_dir().is_dir());
        assert!(home.crash_reports_dir().is_dir());
        assert!(tmp.path().join("templates").is_dir());
    }

    // -----------------------------------------------------------------------
    // config.json custom root
    // -----------------------------------------------------------------------

    #[test]
    fn new_uses_default_when_no_config() {
        // DraffityHome::new() reads from ~/.draffity/ which doesn't exist in CI.
        // We verify that it doesn't panic and returns Some root.
        let home = DraffityHome::new();
        assert!(!home.root().as_os_str().is_empty());
    }

    #[test]
    fn config_json_with_custom_root_overrides_default() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        let custom = tmp.path().join("custom-draffity");
        let config = DraffityConfig {
            root: Some(custom.to_string_lossy().into_owned()),
        };
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        // Set up DraffityHome pointing at tmp, then load config as `new()` would.
        let mut home = DraffityHome::with_root(tmp.path().to_path_buf());
        // Simulate what new() does: load config and override root
        if let Ok(cfg) = DraffityHome::load_config(&home.root) {
            if let Some(custom_root) = cfg.root.filter(|p| !p.is_empty()) {
                home.root = PathBuf::from(custom_root);
            }
        }
        assert_eq!(home.root, custom);
    }

    // -----------------------------------------------------------------------
    // set_root — persists config.json
    // -----------------------------------------------------------------------

    #[test]
    fn set_root_writes_config_and_updates_root() {
        let tmp = tempfile::tempdir().unwrap();
        let mut home = DraffityHome::with_root(tmp.path().to_path_buf());
        let new_root = tmp.path().join("new-root");
        home.set_root(new_root.clone()).unwrap();
        assert_eq!(home.root(), new_root);
        // config.json was written at the original root. Read and verify the custom root.
        let config_path = tmp.path().join("config.json");
        assert!(config_path.exists());
        let contents = std::fs::read_to_string(&config_path).unwrap();
        let decoded: DraffityConfig = serde_json::from_str(&contents).unwrap();
        assert_eq!(decoded.root, Some(new_root.to_string_lossy().to_string()));
    }

    // -----------------------------------------------------------------------
    // Migration
    // -----------------------------------------------------------------------

    #[test]
    fn migration_copies_all_dirs_and_marker() {
        let old = tempfile::tempdir().unwrap();
        let new = tempfile::tempdir().unwrap();

        // Populate old with known files
        std::fs::create_dir_all(old.path().join("voice").join("bin")).unwrap();
        std::fs::create_dir_all(old.path().join("voice").join("models")).unwrap();
        std::fs::write(
            old.path().join("voice").join("bin").join("whisper.exe"),
            b"bin",
        )
        .unwrap();
        std::fs::write(
            old.path().join("voice").join("models").join("ggml.bin"),
            b"model",
        )
        .unwrap();
        std::fs::create_dir_all(old.path().join("media")).unwrap();
        std::fs::write(old.path().join("media").join("img.png"), b"img").unwrap();
        std::fs::write(old.path().join("draffity.db"), b"db").unwrap();

        let home = DraffityHome::with_root(new.path().to_path_buf());
        let migrated = home.run_migration(old.path()).unwrap();
        assert!(migrated, "migration should return true");

        assert!(new
            .path()
            .join("voice")
            .join("bin")
            .join("whisper.exe")
            .exists());
        assert!(new
            .path()
            .join("voice")
            .join("models")
            .join("ggml.bin")
            .exists());
        assert!(new.path().join("media").join("img.png").exists());
        assert!(new.path().join("draffity.db").exists());
        assert!(new.path().join(".migrated-from").exists());
    }

    #[test]
    fn migration_is_idempotent() {
        let old = tempfile::tempdir().unwrap();
        let new = tempfile::tempdir().unwrap();
        std::fs::write(old.path().join("draffity.db"), b"db").unwrap();

        let home = DraffityHome::with_root(new.path().to_path_buf());
        home.run_migration(old.path()).unwrap();
        let second = home.run_migration(old.path()).unwrap();
        assert!(
            !second,
            "second call should return false (already migrated)"
        );
    }

    #[test]
    fn migration_skipped_when_target_already_has_db() {
        let old = tempfile::tempdir().unwrap();
        let new = tempfile::tempdir().unwrap();
        std::fs::write(new.path().join("draffity.db"), b"existing").unwrap();

        let home = DraffityHome::with_root(new.path().to_path_buf());
        let result = home.run_migration(old.path()).unwrap();
        assert!(!result, "should skip because target already has db");
    }
}
