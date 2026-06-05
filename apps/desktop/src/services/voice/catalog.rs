//! Catálogo dinámico de voces TTS: manifest (R2) → cache (~/.draffity) →
//! semilla built-in, agrupado por idioma con los `featured` primero.

use std::collections::HashSet;

use crate::services::voice::registry::{lang_display_name, VoiceManifest};
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
    let mut by_lang: BTreeMap<String, Vec<CatalogVoice>> = BTreeMap::new();
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
    }

    let mk = |lang: &str, voices: Vec<CatalogVoice>, featured: bool| CatalogLang {
        lang: lang.to_string(),
        lang_name: lang_display_name(lang).to_string(),
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
