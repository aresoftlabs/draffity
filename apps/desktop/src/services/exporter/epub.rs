//! EPUB export. Each document becomes one XHTML chapter inside the EPUB
//! container. Folders are still emitted as chapters so the table of contents
//! reflects the project structure 1:1.

use std::io::Cursor;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

use crate::domain::{DocNode, Project};
use crate::error::{AppError, AppResult};

use super::util::{flatten_in_order, xml_escape};

pub fn render(project: &Project, documents: &[DocNode]) -> AppResult<Vec<u8>> {
    let zip = ZipLibrary::new().map_err(|e| AppError::Unexpected(format!("epub zip init: {e}")))?;
    let mut builder = EpubBuilder::new(zip)
        .map_err(|e| AppError::Unexpected(format!("epub builder init: {e}")))?;

    builder
        .metadata("title", &project.title)
        .map_err(|e| AppError::Unexpected(format!("epub metadata: {e}")))?;

    if let Some(meta) = &project.metadata {
        if let Some(author) = meta.get("author").and_then(|v| v.as_str()) {
            let _ = builder.metadata("author", author);
        }
    }

    let cover_html = title_chapter_xhtml(&project.title);
    builder
        .add_content(
            EpubContent::new("cover.xhtml", cover_html.as_bytes())
                .title(&project.title)
                .reftype(ReferenceType::TitlePage),
        )
        .map_err(|e| AppError::Unexpected(format!("epub cover: {e}")))?;

    let ordered = flatten_in_order(documents);
    for (idx, (_depth, doc)) in ordered.iter().enumerate() {
        let filename = format!("ch{:03}.xhtml", idx + 1);
        let xhtml = chapter_xhtml(&doc.title, doc.content.as_deref().unwrap_or(""));
        builder
            .add_content(
                EpubContent::new(filename, xhtml.as_bytes())
                    .title(doc.title.clone())
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| AppError::Unexpected(format!("epub chapter: {e}")))?;
    }

    let mut out = Vec::new();
    let mut cursor = Cursor::new(&mut out);
    builder
        .generate(&mut cursor)
        .map_err(|e| AppError::Unexpected(format!("epub generate: {e}")))?;
    Ok(out)
}

fn title_chapter_xhtml(title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{0}</title></head>
<body><h1>{0}</h1></body>
</html>"#,
        xml_escape(title)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::{doc, project};

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
        let bytes = render(&p, &docs).unwrap();
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
        let bytes = render(&p, &[]).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }
}
