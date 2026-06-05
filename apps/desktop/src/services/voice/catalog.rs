//! Catálogo dinámico de voces TTS: manifest (R2) → cache (~/.draffity) →
//! semilla built-in, agrupado por idioma con los `featured` primero.

use std::collections::HashSet;

use crate::services::voice::registry::{
    lang_display_name, seed_voice_manifest, VoiceManifest, VOICE_MANIFEST_URL,
};
use crate::services::DraffityHome;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVoice {
    pub id: String,
    pub name: String,
    pub lang: String,
    pub quality: String,
    pub size_mb: u32,
    pub recommended: bool,
    pub installed: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogLang {
    pub lang: String,
    pub lang_name: String,
    pub featured: bool,
    pub voices: Vec<CatalogVoice>,
}

pub fn parse_manifest(raw: &str) -> Result<VoiceManifest, serde_json::Error> {
    serde_json::from_str(raw)
}

/// Agrupa por idioma: primero los `featured` (en su orden), luego el resto
/// ordenado alfabéticamente por nombre legible. Puro y testeable.
pub fn build_catalog(m: &VoiceManifest, installed: &HashSet<String>) -> Vec<CatalogLang> {
    use std::collections::BTreeMap;

    // Collect voices by lang; also record the first non-empty lang_name per lang
    // from the manifest so obscure languages display their native name.
    let mut by_lang: BTreeMap<String, Vec<CatalogVoice>> = BTreeMap::new();
    let mut manifest_lang_names: BTreeMap<String, String> = BTreeMap::new();
    for v in &m.voices {
        by_lang
            .entry(v.lang.clone())
            .or_default()
            .push(CatalogVoice {
                id: v.id.clone(),
                name: v.name.clone(),
                lang: v.lang.clone(),
                quality: v.quality.clone(),
                size_mb: v.size_mb,
                recommended: v.recommended,
                installed: installed.contains(&v.id),
            });
        if !v.lang_name.is_empty() {
            manifest_lang_names
                .entry(v.lang.clone())
                .or_insert_with(|| v.lang_name.clone());
        }
    }

    // Resolve display name: manifest value wins, hardcoded fallback otherwise.
    let resolve_name = |lang: &str| -> String {
        manifest_lang_names
            .get(lang)
            .cloned()
            .unwrap_or_else(|| lang_display_name(lang).to_string())
    };

    let mk = |lang: &str, voices: Vec<CatalogVoice>, featured: bool| CatalogLang {
        lang: lang.to_string(),
        lang_name: resolve_name(lang),
        featured,
        voices,
    };

    let mut out: Vec<CatalogLang> = Vec::new();
    for code in &m.featured {
        if let Some(voices) = by_lang.remove(code) {
            out.push(mk(code, voices, true));
        }
    }
    let mut rest: Vec<CatalogLang> = by_lang
        .into_iter()
        .map(|(code, voices)| mk(&code, voices, false))
        .collect();
    rest.sort_by(|a, b| a.lang_name.cmp(&b.lang_name));
    out.extend(rest);
    out
}

/// Path del manifest cacheado.
pub fn cached_manifest_path(home: &DraffityHome) -> std::path::PathBuf {
    home.root().join("cache").join("voice-manifest.json")
}

/// Lee el manifest cacheado; si falta o está corrupto, devuelve la semilla.
pub fn load_cached_or_seed(home: &DraffityHome) -> VoiceManifest {
    let path = cached_manifest_path(home);
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(m) = parse_manifest(&raw) {
            return m;
        }
    }
    seed_voice_manifest()
}

/// Refresca el cache desde R2 (best-effort). Errores de red se ignoran: el
/// llamador sigue con cache/semilla. Bloqueante (correr en spawn_blocking).
pub fn refresh_manifest_cache(home: &DraffityHome) {
    let resp = match reqwest::blocking::Client::new()
        .get(VOICE_MANIFEST_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
    {
        Ok(r) if r.status().is_success() => r,
        _ => return,
    };
    let body = match resp.text() {
        Ok(b) => b,
        Err(_) => return,
    };
    if parse_manifest(&body).is_ok() {
        let path = cached_manifest_path(home);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::voice::registry::{ManifestVoice, VoiceManifest};

    fn voice(id: &str, lang: &str) -> ManifestVoice {
        ManifestVoice {
            id: id.into(),
            name: id.into(),
            lang: lang.into(),
            lang_name: String::new(),
            locale: String::new(),
            quality: "medium".into(),
            size_mb: 60,
            onnx_url: "u".into(),
            config_url: "c".into(),
            onnx_md5: None,
            config_md5: None,
            recommended: false,
        }
    }

    #[test]
    fn build_catalog_puts_featured_langs_first() {
        let m = VoiceManifest {
            schema_version: 1,
            featured: vec!["es".into(), "en".into(), "pt".into()],
            voices: vec![
                voice("de_x", "de"),
                voice("es_x", "es"),
                voice("en_x", "en"),
            ],
        };
        let installed = std::collections::HashSet::new();
        let cat = build_catalog(&m, &installed);
        let langs: Vec<&str> = cat.iter().map(|g| g.lang.as_str()).collect();
        assert_eq!(langs, vec!["es", "en", "de"]);
        assert!(cat[0].featured);
        assert!(!cat.last().unwrap().featured);
    }

    #[test]
    fn build_catalog_marks_installed() {
        let m = VoiceManifest {
            schema_version: 1,
            featured: vec![],
            voices: vec![voice("es_x", "es")],
        };
        let installed: std::collections::HashSet<String> = ["es_x".to_string()].into();
        let cat = build_catalog(&m, &installed);
        assert!(cat[0].voices[0].installed);
    }

    fn voice_with_lang_name(id: &str, lang: &str, lang_name: &str) -> ManifestVoice {
        ManifestVoice {
            lang_name: lang_name.into(),
            ..voice(id, lang)
        }
    }

    #[test]
    fn build_catalog_uses_manifest_lang_name_when_present() {
        let m = VoiceManifest {
            schema_version: 1,
            featured: vec![],
            voices: vec![
                voice_with_lang_name("fi_x", "fi", "suomi"),
                // second voice for the same lang should NOT override the first
                voice_with_lang_name("fi_y", "fi", "should-be-ignored"),
            ],
        };
        let installed = std::collections::HashSet::new();
        let cat = build_catalog(&m, &installed);
        assert_eq!(cat[0].lang, "fi");
        assert_eq!(cat[0].lang_name, "suomi");
    }

    #[test]
    fn build_catalog_falls_back_to_hardcoded_when_manifest_lang_name_empty() {
        let m = VoiceManifest {
            schema_version: 1,
            featured: vec![],
            // lang_name is "" (default from voice())
            voices: vec![voice("de_x", "de")],
        };
        let installed = std::collections::HashSet::new();
        let cat = build_catalog(&m, &installed);
        assert_eq!(cat[0].lang, "de");
        assert_eq!(cat[0].lang_name, "Deutsch");
    }

    #[test]
    fn load_manifest_prefers_cache_then_seed() {
        use crate::services::DraffityHome;
        let dir = tempfile::tempdir().unwrap();
        let home = DraffityHome::with_root(dir.path().to_path_buf());

        // Sin cache ⇒ semilla (2 voces).
        let m = super::load_cached_or_seed(&home);
        assert_eq!(m.voices.len(), 2);

        // Con cache válido ⇒ usa cache.
        let path = super::cached_manifest_path(&home);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"{"schemaVersion":1,"featured":[],"voices":[
            {"id":"x","name":"X","lang":"es","sizeMb":1,"onnxUrl":"o","configUrl":"c"}]}"#,
        )
        .unwrap();
        let m2 = super::load_cached_or_seed(&home);
        assert_eq!(m2.voices.len(), 1);
        assert_eq!(m2.voices[0].id, "x");

        // Cache corrupto ⇒ vuelve a la semilla.
        std::fs::write(&path, "garbage").unwrap();
        let m3 = super::load_cached_or_seed(&home);
        assert_eq!(m3.voices.len(), 2);
    }

    #[test]
    fn parse_manifest_rejects_garbage_and_accepts_valid() {
        assert!(parse_manifest("not json").is_err());
        let json = r#"{"schemaVersion":1,"featured":["es"],"voices":[
            {"id":"es_ES-davefx-medium","name":"Dave","lang":"es","sizeMb":63,
             "onnxUrl":"o","configUrl":"c"}]}"#;
        let m = parse_manifest(json).unwrap();
        assert_eq!(m.voices.len(), 1);
        assert_eq!(m.voices[0].id, "es_ES-davefx-medium");
    }
}
