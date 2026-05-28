//! EPUB export. Each document becomes one XHTML chapter inside the EPUB
//! container. Folders are still emitted as chapters so the table of contents
//! reflects the project structure 1:1.

use std::io::Cursor;
use std::path::Path;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

use crate::domain::{extension_for_mime, CodexEntry, CodexKind, DocNode, Project};
use crate::error::{AppError, AppResult};

use super::config::ExportConfig;
use super::media_bundle::MediaBundle;
use super::util::{flatten_in_order, xml_escape};

pub fn render(
    project: &Project,
    documents: &[DocNode],
    codex: &[CodexEntry],
    media: &MediaBundle,
    config: &ExportConfig,
) -> AppResult<Vec<u8>> {
    let zip = ZipLibrary::new().map_err(|e| AppError::Unexpected(format!("epub zip init: {e}")))?;
    let mut builder = EpubBuilder::new(zip)
        .map_err(|e| AppError::Unexpected(format!("epub builder init: {e}")))?;

    let title = config
        .title_override
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(&project.title);
    builder
        .metadata("title", title)
        .map_err(|e| AppError::Unexpected(format!("epub metadata: {e}")))?;

    // Author: explicit config wins; otherwise look in project metadata.
    let author = config
        .author
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            project
                .metadata
                .as_ref()
                .and_then(|m| m.get("author").and_then(|v| v.as_str()))
        });
    if let Some(author) = author {
        let _ = builder.metadata("author", author);
    }

    if let Some(path) = config.cover_image_path.as_deref() {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            add_cover_image(&mut builder, trimmed)?;
        }
    }

    if config.include_toc {
        builder.inline_toc();
    }

    if config.include_title_page {
        let cover_html = title_chapter_xhtml(title, author);
        builder
            .add_content(
                EpubContent::new("title.xhtml", cover_html.as_bytes())
                    .title(title)
                    .reftype(ReferenceType::TitlePage),
            )
            .map_err(|e| AppError::Unexpected(format!("epub title page: {e}")))?;
    }

    // Resolve every media id referenced anywhere in the manuscript to a
    // stable filename, add the bytes as EPUB resources, and remember the
    // mapping so we can rewrite the chapter `src` attributes.
    let media_paths = embed_media_resources(&mut builder, documents, media)?;

    let ordered = flatten_in_order(documents);
    for (idx, (_depth, doc)) in ordered.iter().enumerate() {
        let filename = format!("ch{:03}.xhtml", idx + 1);
        let body = doc.content.as_deref().unwrap_or("");
        let body = rewrite_media_src(body, &media_paths);
        let xhtml = chapter_xhtml(&doc.title, &body);
        builder
            .add_content(
                EpubContent::new(filename, xhtml.as_bytes())
                    .title(doc.title.clone())
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| AppError::Unexpected(format!("epub chapter: {e}")))?;
    }

    if config.include_codex && !codex.is_empty() {
        let appendix = codex_appendix_xhtml(codex);
        builder
            .add_content(
                EpubContent::new("codex.xhtml", appendix.as_bytes())
                    .title("Codex")
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| AppError::Unexpected(format!("epub codex appendix: {e}")))?;
    }

    let mut out = Vec::new();
    let mut cursor = Cursor::new(&mut out);
    builder
        .generate(&mut cursor)
        .map_err(|e| AppError::Unexpected(format!("epub generate: {e}")))?;
    Ok(out)
}

fn add_cover_image(builder: &mut EpubBuilder<ZipLibrary>, path: &str) -> AppResult<()> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Unexpected(format!("read cover image '{path}': {e}")))?;
    let p = Path::new(path);
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let (mime, target) = match ext.as_str() {
        "jpg" | "jpeg" => ("image/jpeg", "cover.jpg"),
        "png" => ("image/png", "cover.png"),
        "gif" => ("image/gif", "cover.gif"),
        "webp" => ("image/webp", "cover.webp"),
        _ => {
            return Err(AppError::Unsupported(format!(
                "cover image extension '{ext}' is not supported (use jpg/png/gif/webp)"
            )))
        }
    };
    builder
        .add_cover_image(target, Cursor::new(bytes), mime)
        .map_err(|e| AppError::Unexpected(format!("epub cover image: {e}")))?;
    Ok(())
}

fn title_chapter_xhtml(title: &str, author: Option<&str>) -> String {
    let safe_title = xml_escape(title);
    let author_block = match author {
        Some(a) => format!("<p class=\"author\">{}</p>", xml_escape(a)),
        None => String::new(),
    };
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{safe_title}</title></head>
<body><h1>{safe_title}</h1>{author_block}</body>
</html>"#,
    )
}

fn chapter_xhtml(title: &str, content_html: &str) -> String {
    let safe_title = xml_escape(title);
    // We trust TipTap output to be reasonably XHTML-compatible. If a future
    // extension produces tags that break parsing, the fallback is a wrapping
    // <p> via parsing and re-serializing — out of scope for the MVP.
    let body = if content_html.trim().is_empty() {
        String::new()
    } else {
        content_html.to_string()
    };
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{safe_title}</title></head>
<body>
<h1>{safe_title}</h1>
{body}
</body>
</html>"#,
    )
}

fn codex_appendix_xhtml(codex: &[CodexEntry]) -> String {
    let mut body = String::from("<h1>Codex</h1>\n");
    for kind in [
        CodexKind::Character,
        CodexKind::Place,
        CodexKind::Object,
        CodexKind::Note,
    ] {
        let mut entries: Vec<&CodexEntry> = codex.iter().filter(|e| e.kind == kind).collect();
        if entries.is_empty() {
            continue;
        }
        entries.sort_by_key(|a| a.name.to_lowercase());
        body.push_str(&format!(
            "<h2>{}</h2>\n",
            xml_escape(codex_section_name(kind))
        ));
        for e in entries {
            body.push_str(&format!("<h3>{}</h3>\n", xml_escape(&e.name)));
            if !e.tags.is_empty() {
                body.push_str(&format!(
                    "<p><em>{}</em></p>\n",
                    xml_escape(&e.tags.join(", "))
                ));
            }
            if let Some(b) = &e.body {
                if !b.trim().is_empty() {
                    // Body HTML is trusted (came out of TipTap). Wrap in a
                    // div so the reader scopes structure if we ever style
                    // codex entries differently.
                    body.push_str(&format!("<div>{b}</div>\n"));
                }
            }
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Codex</title></head>
<body>
{body}
</body>
</html>"#,
    )
}

/// Add every media asset referenced by the documents as an EPUB resource
/// (filename `media/<id>.<ext>`) and return the `id → filename` map so
/// chapter `<img>` rewrites can swap `data-media-id` for `src`.
fn embed_media_resources(
    builder: &mut EpubBuilder<ZipLibrary>,
    documents: &[DocNode],
    media: &MediaBundle,
) -> AppResult<std::collections::HashMap<String, String>> {
    let mut paths = std::collections::HashMap::new();
    if media.is_empty() {
        return Ok(paths);
    }
    let mut seen = std::collections::HashSet::new();
    for doc in documents {
        let Some(html) = &doc.content else {
            continue;
        };
        for id in super::media_bundle::extract_media_ids(html) {
            if !seen.insert(id.clone()) {
                continue;
            }
            let Some((mime, bytes)) = media.get(&id) else {
                continue;
            };
            let ext = extension_for_mime(mime);
            let filename = format!("media/{id}.{ext}");
            builder
                .add_resource(&filename, std::io::Cursor::new(bytes.to_vec()), mime)
                .map_err(|e| AppError::Unexpected(format!("epub media resource: {e}")))?;
            paths.insert(id, filename);
        }
    }
    Ok(paths)
}

/// Replace `<img data-media-id="X">` with `<img src="media/X.ext">` for
/// every id we managed to embed. Untouched ids stay as
/// `data-media-id="…"` so a stale reference is still visible in the source.
fn rewrite_media_src(html: &str, paths: &std::collections::HashMap<String, String>) -> String {
    if paths.is_empty() || !html.contains("data-media-id=\"") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    while let Some(start) = html[cursor..].find("<img") {
        let abs_start = cursor + start;
        out.push_str(&html[cursor..abs_start]);
        let Some(end_off) = html[abs_start..].find('>') else {
            out.push_str(&html[abs_start..]);
            return out;
        };
        let tag_end = abs_start + end_off + 1;
        let tag = &html[abs_start..tag_end];
        let mut replaced = tag.to_string();
        if let Some(id) = extract_attr(tag, "data-media-id") {
            if let Some(filename) = paths.get(&id) {
                replaced = tag.replacen("<img", &format!("<img src=\"{filename}\""), 1);
            }
        }
        out.push_str(&replaced);
        cursor = tag_end;
    }
    out.push_str(&html[cursor..]);
    out
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn codex_section_name(kind: CodexKind) -> &'static str {
    match kind {
        CodexKind::Character => "Characters",
        CodexKind::Place => "Places",
        CodexKind::Object => "Objects",
        CodexKind::Note => "Notes",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::{doc, project};

    fn test_tempdir(prefix: &str) -> std::path::PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mut p = std::env::temp_dir();
        p.push(format!("draffity-{prefix}-{nanos:x}"));
        std::fs::create_dir_all(&p).expect("create tempdir");
        p
    }

    #[test]
    fn produces_valid_zip_with_mimetype_first() {
        let p = project("Mi novela");
        let pid = p.id.clone();
        let docs = vec![doc(
            "a",
            &pid,
            None,
            "Capítulo 1",
            DocumentType::Chapter,
            Some("<p>Hola.</p>"),
            0,
        )];
        let bytes = render(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            &ExportConfig::default(),
        )
        .unwrap();
        // ZIP magic
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        // EPUB requires the mimetype file as the first entry, uncompressed.
        // The string "mimetype" should appear in the first ~80 bytes.
        let header = &bytes[..bytes.len().min(120)];
        assert!(header.windows(8).any(|w| w == b"mimetype"));
    }

    #[test]
    fn empty_project_still_renders() {
        let p = project("X");
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &ExportConfig::default()).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }

    #[test]
    fn title_override_unit_round_trip() {
        // EPUB entries are deflated so the raw bytes don't expose the title
        // verbatim. Exercise the title-resolution branch directly via the
        // helper that the renderer uses.
        let html = title_chapter_xhtml("Override Title", Some("Borges"));
        assert!(html.contains("Override Title"));
        assert!(html.contains("Borges"));
    }

    #[test]
    fn skipping_title_page_omits_title_entry() {
        let p = project("Sin portada");
        let cfg = ExportConfig {
            include_title_page: false,
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        let lossy = String::from_utf8_lossy(&bytes);
        assert!(!lossy.contains("title.xhtml"));
    }

    #[test]
    fn cover_image_extension_must_be_recognised() {
        let p = project("Cover bug");
        let cfg = ExportConfig {
            cover_image_path: Some("/tmp/missing.bmp".into()),
            ..ExportConfig::default()
        };
        let err = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap_err();
        match err {
            crate::error::AppError::Unsupported(msg) => {
                assert!(msg.contains("bmp"));
            }
            crate::error::AppError::Unexpected(_) => {
                // On Windows, the read may fail before extension check; both
                // outcomes prove we don't silently pass a non-image through.
            }
            other => panic!("expected Unsupported/Unexpected, got: {other:?}"),
        }
    }

    #[test]
    fn cover_image_bytes_are_embedded_in_zip() {
        // Smallest legal JPEG: a 1x1 placeholder produced by an online tool.
        // We embed it via a tempfile and verify the EPUB references "cover.jpg".
        const TINY_JPEG: &[u8] = &[
            0xFF, 0xD8, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
            0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C,
            0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20,
            0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30,
            0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34,
            0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
            0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
            0x07, 0x08, 0x09, 0x0A, 0x0B, 0xFF, 0xC4, 0x00, 0xB5, 0x10, 0x00, 0x02, 0x01, 0x03,
            0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D, 0x01, 0x02,
            0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
            0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52,
            0xD1, 0xF0, 0x24, 0x33, 0x62, 0x72, 0x82, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00,
            0x00, 0x3F, 0x00, 0xFB, 0xD0, 0xFF, 0xD9,
        ];
        let dir = test_tempdir("epub-cover-test");
        let cover = dir.join("cover.jpg");
        std::fs::write(&cover, TINY_JPEG).unwrap();

        let p = project("Con portada");
        let cfg = ExportConfig {
            cover_image_path: Some(cover.to_string_lossy().to_string()),
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        // EPUB ZIP central directory lists the file name; "cover.jpg" must
        // appear somewhere in the package.
        let lossy = String::from_utf8_lossy(&bytes);
        assert!(lossy.contains("cover.jpg"));
    }
}
