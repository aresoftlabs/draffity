//! Media payload passed into the export pipeline. Pre-resolved at the
//! command layer so renderers don't need a handle to `MediaService` —
//! the trait `ExportService` stays oblivious to storage.

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct MediaBundle {
    /// `media_id → (mime, bytes)`.
    entries: HashMap<String, (String, Vec<u8>)>,
}

impl MediaBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: impl Into<String>, mime: impl Into<String>, bytes: Vec<u8>) {
        self.entries.insert(id.into(), (mime.into(), bytes));
    }

    pub fn get(&self, id: &str) -> Option<(&str, &[u8])> {
        self.entries
            .get(id)
            .map(|(mime, bytes)| (mime.as_str(), bytes.as_slice()))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate `(id, mime, bytes)` in insertion-stable order. Used by EPUB
    /// to add resources before rewriting the chapter `src`s.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str, &[u8])> {
        self.entries
            .iter()
            .map(|(id, (mime, bytes))| (id.as_str(), mime.as_str(), bytes.as_slice()))
    }
}

/// Walk HTML strings for `data-media-id="…"` references. Used by the
/// command layer to figure out which assets to pre-load into the
/// `MediaBundle`. Naive substring search: good enough since the
/// attribute pattern is deterministic in the persisted TipTap output.
pub fn extract_media_ids(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let needle = "data-media-id=\"";
    let mut cursor = 0;
    while let Some(start) = html[cursor..].find(needle) {
        let abs_start = cursor + start + needle.len();
        let rest = &html[abs_start..];
        if let Some(end) = rest.find('"') {
            let id = &rest[..end];
            if !id.is_empty() {
                out.push(id.to_string());
            }
            cursor = abs_start + end + 1;
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_finds_single_id() {
        let html = r#"<p>Hello</p><img data-media-id="abc123" alt="x"/><p>tail</p>"#;
        assert_eq!(extract_media_ids(html), vec!["abc123"]);
    }

    #[test]
    fn extract_finds_multiple_ids_in_order() {
        let html = r#"<img data-media-id="a"/><p>...</p><img data-media-id="b" alt="x">"#;
        assert_eq!(extract_media_ids(html), vec!["a", "b"]);
    }

    #[test]
    fn extract_skips_empty_ids() {
        let html = r#"<img data-media-id="" alt="x">"#;
        assert!(extract_media_ids(html).is_empty());
    }

    #[test]
    fn extract_returns_empty_for_html_without_images() {
        assert!(extract_media_ids("<p>no images here</p>").is_empty());
    }

    #[test]
    fn bundle_round_trips_get() {
        let mut b = MediaBundle::new();
        b.insert("id1", "image/png", vec![1, 2, 3]);
        let (mime, bytes) = b.get("id1").unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(bytes, &[1, 2, 3]);
        assert!(b.get("missing").is_none());
    }
}
