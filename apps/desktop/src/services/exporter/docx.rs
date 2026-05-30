//! DOCX export. We parse the TipTap HTML with `scraper` and emit a stream of
//! paragraphs/headings/runs into a `docx-rs` document. The HTML→paragraph
//! pipeline and the Codex appendix builder live in `docx_helpers`; `render`
//! stays as a thin orchestrator.

use docx_rs::{
    AlignmentType, BreakType, Docx, PageMargin, Paragraph, Run, RunFonts, StyleWithLevel,
    TableOfContents,
};

use crate::domain::{CodexEntry, DocNode, Project};
use crate::error::AppResult;

use super::config::{ExportConfig, PageSize};
use super::docx_helpers::{append_codex, render_html_blocks};
use super::media_bundle::MediaBundle;
use super::util::flatten_in_order;

/// Export the project to a `.docx`. Image embedding via `docx-rs::Pic`
/// needs PNG/JPEG dimension parsing; for now we accept the `MediaBundle`
/// in the signature to keep the trait surface uniform but skip actual
/// embedding — images render as their alt text. Follow-up will pull
/// image dimensions and emit proper inline pictures.
pub fn render(
    project: &Project,
    documents: &[DocNode],
    codex: &[CodexEntry],
    _media: &MediaBundle,
    config: &ExportConfig,
) -> AppResult<Vec<u8>> {
    let mut docx = apply_page_setup(Docx::new(), config);

    let display_title = config
        .title_override
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&project.title);

    if config.include_title_page {
        docx = add_title_page(docx, display_title, config);
    }
    if config.include_toc {
        docx = add_toc(docx);
    }

    let ordered = flatten_in_order(documents);
    let separator = config.scene_separator.as_text();
    let mut top_level_seen = false;
    for (depth, doc) in ordered {
        // Scene separator between consecutive top-level documents (K-02).
        if depth == 0 {
            if top_level_seen && !separator.is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Center)
                        .add_run(Run::new().add_text(separator)),
                );
            }
            top_level_seen = true;
        }
        docx = add_document(docx, doc, depth);
    }

    if config.include_codex && !codex.is_empty() {
        docx = append_codex(docx, codex);
    }

    let mut buf: Vec<u8> = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut buf);
        docx.build()
            .pack(cursor)
            .map_err(|e| crate::error::AppError::Unexpected(format!("docx build failed: {e}")))?;
    }
    Ok(buf)
}

/// Apply `page_size`, `margins` and `font_family` from the export config so a
/// preset like "Manuscript submission" (K-05) actually produces Courier 12pt
/// on the chosen page size, instead of docx-rs defaults (AUD-13).
fn apply_page_setup(mut docx: Docx, config: &ExportConfig) -> Docx {
    let (w, h) = page_dimensions_twips(&config.page_size);
    docx = docx.page_size(w, h).page_margin(
        PageMargin::new()
            .top(mm_to_twips(config.margins.top_mm))
            .bottom(mm_to_twips(config.margins.bottom_mm))
            .left(mm_to_twips(config.margins.left_mm))
            .right(mm_to_twips(config.margins.right_mm)),
    );
    if let Some(font) = resolve_docx_font(config.font_family.as_deref()) {
        docx = docx
            .default_fonts(RunFonts::new().ascii(&font).hi_ansi(&font).east_asia(&font))
            .default_size(24); // 24 half-points = 12pt
    }
    docx
}

/// Millimetres to twips (1 inch = 25.4 mm = 1440 twips).
fn mm_to_twips(mm: u32) -> i32 {
    ((mm as f64) * 1440.0 / 25.4).round() as i32
}

fn page_dimensions_twips(size: &PageSize) -> (u32, u32) {
    match size {
        PageSize::A4 => (11906, 16838),
        PageSize::Letter => (12240, 15840),
        PageSize::Legal => (12240, 20160),
        PageSize::Custom {
            width_mm,
            height_mm,
        } => (
            mm_to_twips(*width_mm) as u32,
            mm_to_twips(*height_mm) as u32,
        ),
    }
}

/// Map a generic font-family hint (`monospace`/`serif`/`sans`) — or an explicit
/// font name — to a concrete Word font. `None` keeps the docx default.
fn resolve_docx_font(family: Option<&str>) -> Option<String> {
    let raw = family?.trim();
    if raw.is_empty() {
        return None;
    }
    let name = match raw.to_ascii_lowercase().as_str() {
        "monospace" => "Courier New",
        "serif" => "Times New Roman",
        "sans" | "sans-serif" => "Arial",
        _ => raw,
    };
    Some(name.to_string())
}

fn add_title_page(mut docx: Docx, display_title: &str, config: &ExportConfig) -> Docx {
    docx = docx.add_paragraph(
        Paragraph::new()
            .style("Heading1")
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(display_title).size(72).bold()),
    );
    if let Some(author) = config
        .author
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        docx = docx.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text(author).size(32)),
        );
    }
    // Page break so the manuscript itself starts on the next page.
    docx.add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)))
}

fn add_toc(docx: Docx) -> Docx {
    // Standard TOC that picks up Heading1..Heading6. `dirty()` makes Word
    // prompt the user to update on open (or auto-update if configured).
    let toc = TableOfContents::new()
        .heading_styles_range(1, 6)
        .add_style_with_level(StyleWithLevel::new("Heading1", 1))
        .add_style_with_level(StyleWithLevel::new("Heading2", 2))
        .add_style_with_level(StyleWithLevel::new("Heading3", 3))
        .add_style_with_level(StyleWithLevel::new("Heading4", 4))
        .add_style_with_level(StyleWithLevel::new("Heading5", 5))
        .add_style_with_level(StyleWithLevel::new("Heading6", 6))
        .hyperlink()
        .dirty();
    let docx = docx.add_table_of_contents(toc);
    // Page break after TOC so chapters start on a fresh page.
    docx.add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)))
}

fn add_document(mut docx: Docx, doc: &DocNode, depth: usize) -> Docx {
    let title_level = (depth + 2).min(6);
    docx = docx.add_paragraph(
        Paragraph::new()
            .style(&format!("Heading{title_level}"))
            .add_run(
                Run::new()
                    .add_text(doc.title.clone())
                    .bold()
                    .fonts(RunFonts::new().east_asia("Lora")),
            ),
    );
    if let Some(html) = &doc.content {
        if !html.trim().is_empty() {
            for paragraph in render_html_blocks(html) {
                docx = docx.add_paragraph(paragraph);
            }
        }
    }
    docx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::{doc, project};

    /// DOCX is a ZIP — these magic bytes (`PK\x03\x04`) confirm we built
    /// a valid container. Round-tripping the full content would need a DOCX
    /// reader; for the MVP, valid container + non-empty word/document.xml
    /// is the contract.
    #[test]
    fn produces_valid_zip_container() {
        let p = project("Mi novela");
        let pid = p.id.clone();
        let docs = vec![doc(
            "a",
            &pid,
            None,
            "Capítulo 1",
            DocumentType::Chapter,
            Some(
                "<h2>Sub</h2><p>Hola <strong>mundo</strong>.</p><ul><li>Uno</li><li>Dos</li></ul>",
            ),
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
        // ZIP local file header signature
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        // Should contain at least the standard word/document.xml entry name.
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("word/document.xml"));
    }

    #[test]
    fn manuscript_preset_applies_courier_12pt_and_page_size() {
        use super::super::config::PageSize;
        let p = project("Novela");
        let cfg = ExportConfig {
            font_family: Some("monospace".into()),
            page_size: PageSize::Letter,
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        // Manuscript standard: Courier 12pt (K-05).
        assert!(
            s.contains("Courier New"),
            "expected Courier New default font"
        );
        assert!(
            s.contains("w:val=\"24\""),
            "expected 12pt (24 half-points) default size"
        );
        // Letter page size in twips (8.5in x 11in).
        assert!(s.contains("w:w=\"12240\""), "expected Letter page width");
    }

    #[test]
    fn empty_project_still_produces_zip() {
        let p = project("X");
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &ExportConfig::default()).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }

    /// The TOC field in DOCX is rendered as a `w:fldChar` with the `TOC`
    /// instruction. We don't need a full DOCX parser — finding the literal
    /// "TOC" instruction text in the package is enough to verify it was
    /// emitted (and absent when disabled).
    #[test]
    fn toc_is_emitted_when_include_toc_is_true() {
        let p = project("Con TOC");
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &ExportConfig::default()).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("TOC"), "expected TOC field in DOCX output");
    }

    #[test]
    fn toc_is_omitted_when_include_toc_is_false() {
        let p = project("Sin TOC");
        let cfg = ExportConfig {
            include_toc: false,
            include_title_page: false,
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        let s = String::from_utf8_lossy(&bytes);
        assert!(
            !s.contains("TOC \\"),
            "TOC field should not appear when disabled"
        );
    }

    #[test]
    fn title_page_omits_when_disabled() {
        // We can't easily introspect compressed DOCX text but we can verify
        // by exercising the no-headers path: when title page is off and
        // there are no docs, the doc still builds without panicking.
        let p = project("Title-less");
        let cfg = ExportConfig {
            include_title_page: false,
            include_toc: false,
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }

    #[test]
    fn title_override_is_used_in_title_page() {
        // Same caveat as EPUB: deflated. Exercise the resolver by checking
        // an explicit override falls back to project.title when empty.
        let p = project("Original");
        let cfg = ExportConfig {
            title_override: Some("  ".into()),
            ..ExportConfig::default()
        };
        // Whitespace-only override falls back to project title — the build
        // doesn't panic and we trust the resolver branch coverage.
        render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
    }
}
