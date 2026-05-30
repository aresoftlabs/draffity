//! PDF export. We don't ship a PDF renderer ourselves — instead we emit a
//! standalone HTML file with print-friendly CSS (`page-break-before` per
//! chapter, conservative margins, Times-style serif default) and let the
//! UI hand it off to the OS print dialog ("Save as PDF" is the common
//! exit). This keeps the dependency tree light and the layout WYSIWYG
//! with what the user sees in the editor.
//!
//! The HTML is self-contained — images embed as base64 data URIs (same
//! pipeline Markdown uses), footnotes render inline with a Notes section
//! per chapter, and the Codex appendix shows up when enabled.

use crate::domain::{CodexEntry, CodexKind, DocNode, Project};
use crate::error::AppResult;

use super::config::ExportConfig;
use super::footnotes::{collect_footnotes, Footnote};
use super::media_bundle::MediaBundle;
use super::util::{flatten_in_order, xml_escape};

pub fn render(
    project: &Project,
    documents: &[DocNode],
    codex: &[CodexEntry],
    media: &MediaBundle,
    config: &ExportConfig,
) -> AppResult<Vec<u8>> {
    let display_title = config
        .title_override
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&project.title);

    let author = config
        .author
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            project
                .metadata
                .as_ref()
                .and_then(|m| m.get("author").and_then(|v| v.as_str()))
        });

    let mut body = String::new();
    if config.include_title_page {
        body.push_str(&format!(
            "<section class=\"title-page\"><h1>{}</h1>{}</section>\n",
            xml_escape(display_title),
            author
                .map(|a| format!("<p class=\"author\">{}</p>", xml_escape(a)))
                .unwrap_or_default()
        ));
    }

    if config.include_toc {
        body.push_str(&render_toc(documents));
    }

    let separator = config.scene_separator.as_text();
    let mut top_level_seen = false;
    for (depth, doc) in flatten_in_order(documents) {
        // Scene separator between consecutive top-level documents (K-02).
        if depth == 0 {
            if top_level_seen && !separator.is_empty() {
                body.push_str(&format!(
                    "<p class=\"scene-separator\">{}</p>\n",
                    xml_escape(separator)
                ));
            }
            top_level_seen = true;
        }
        let level = (depth + 2).min(6);
        body.push_str(&format!(
            "<section class=\"chapter\"><h{level}>{}</h{level}>\n",
            xml_escape(&doc.title)
        ));
        if let Some(html) = &doc.content {
            if !html.trim().is_empty() {
                let (with_refs, notes) =
                    collect_footnotes(html, |n| format!("<sup class=\"fn-ref\">{n}</sup>"));
                let with_media = inline_media(&with_refs, media);
                body.push_str(&with_media);
                if !notes.is_empty() {
                    body.push_str(&render_notes(&notes));
                }
            }
        }
        body.push_str("</section>\n");
    }

    if config.include_codex && !codex.is_empty() {
        body.push_str(&render_codex(codex));
    }

    let html = wrap_document(display_title, author, &body);
    Ok(html.into_bytes())
}

fn render_toc(documents: &[DocNode]) -> String {
    let mut out = String::from("<nav class=\"toc\"><h2>Contents</h2><ol>\n");
    for (depth, doc) in flatten_in_order(documents) {
        let indent = "  ".repeat(depth);
        out.push_str(&format!("{indent}<li>{}</li>\n", xml_escape(&doc.title)));
    }
    out.push_str("</ol></nav>\n");
    out
}

fn render_notes(notes: &[Footnote]) -> String {
    let mut out = String::from("<section class=\"notes\"><hr/><ol>\n");
    for n in notes {
        out.push_str(&format!("<li>{}</li>\n", xml_escape(&n.content)));
    }
    out.push_str("</ol></section>\n");
    out
}

fn render_codex(codex: &[CodexEntry]) -> String {
    let mut body = String::from("<section class=\"codex\"><h2>Codex</h2>\n");
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
            "<h3>{}</h3>\n",
            xml_escape(codex_section_name(kind))
        ));
        for e in entries {
            body.push_str(&format!("<h4>{}</h4>\n", xml_escape(&e.name)));
            if !e.tags.is_empty() {
                body.push_str(&format!(
                    "<p class=\"tags\"><em>{}</em></p>\n",
                    xml_escape(&e.tags.join(", "))
                ));
            }
            if let Some(b) = &e.body {
                if !b.trim().is_empty() {
                    body.push_str(b);
                }
            }
        }
    }
    body.push_str("</section>\n");
    body
}

fn codex_section_name(kind: CodexKind) -> &'static str {
    match kind {
        CodexKind::Character => "Characters",
        CodexKind::Place => "Places",
        CodexKind::Object => "Objects",
        CodexKind::Note => "Notes",
    }
}

fn wrap_document(title: &str, author: Option<&str>, body: &str) -> String {
    let safe_title = xml_escape(title);
    let meta_author = author
        .map(|a| format!("<meta name=\"author\" content=\"{}\">\n", xml_escape(a)))
        .unwrap_or_default();
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{safe_title}</title>
{meta_author}<style>
{print_css}
</style>
</head>
<body>
{body}
<script>
// Auto-trigger the system print dialog when the document loads in a
// browser/webview. The user can pick "Save as PDF" from there. Safe to
// no-op outside a browser context.
window.addEventListener('load', () => {{
  setTimeout(() => {{ try {{ window.print(); }} catch (_e) {{}} }}, 250);
}});
</script>
</body>
</html>
"#,
        print_css = PRINT_CSS,
    )
}

const PRINT_CSS: &str = r#"
@page { size: A4; margin: 24mm 20mm; }
body { font-family: 'Lora', Georgia, 'Times New Roman', serif; font-size: 12pt; line-height: 1.55; color: #111; max-width: 720px; margin: 0 auto; padding: 24px; }
h1 { font-size: 28pt; text-align: center; margin: 0 0 0.5em; }
h2 { font-size: 18pt; margin: 1em 0 0.4em; }
h3 { font-size: 14pt; margin: 1em 0 0.4em; }
p { margin: 0 0 0.8em; text-align: justify; }
.title-page { page-break-after: always; text-align: center; padding-top: 30vh; }
.title-page .author { font-size: 14pt; margin-top: 1em; opacity: 0.8; }
.toc { page-break-after: always; }
.toc ol { list-style: none; padding: 0; }
.toc li { padding: 0.15em 0; border-bottom: 1px dotted #ccc; }
.chapter { page-break-before: always; }
.notes { font-size: 10pt; margin-top: 2em; color: #444; }
.notes hr { border: 0; border-top: 1px solid #999; width: 30%; margin: 1em 0; }
.notes ol { padding-left: 1.4em; }
.codex { page-break-before: always; }
sup.fn-ref { font-size: 0.7em; color: #1d4ed8; }
img { max-width: 100%; height: auto; }
@media print {
  body { padding: 0; }
}
"#;

/// Reuse the markdown exporter's approach to data-URI image inlining so
/// the printed page renders images even when the saved HTML is moved
/// away from its source files.
fn inline_media(html: &str, media: &MediaBundle) -> String {
    use base64::Engine;
    if media.is_empty() || !html.contains("data-media-id=\"") {
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
            if let Some((mime, bytes)) = media.get(&id) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                let data_uri = format!("data:{mime};base64,{b64}");
                replaced = tag.replacen("<img", &format!("<img src=\"{data_uri}\""), 1);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::{doc, project};

    #[test]
    fn produces_standalone_html_with_print_css() {
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
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("Mi novela"));
        assert!(html.contains("Capítulo 1"));
        // Print CSS keys present.
        assert!(html.contains("page-break-before"));
        assert!(html.contains("@page"));
        // Auto print trigger embedded.
        assert!(html.contains("window.print()"));
    }

    #[test]
    fn skips_toc_when_disabled() {
        let p = project("X");
        let cfg = ExportConfig {
            include_toc: false,
            include_title_page: false,
            ..ExportConfig::default()
        };
        let bytes = render(&p, &[], &[], &MediaBundle::new(), &cfg).unwrap();
        let html = String::from_utf8(bytes).unwrap();
        assert!(!html.contains("class=\"toc\""));
        assert!(!html.contains("class=\"title-page\""));
    }

    #[test]
    fn renders_footnotes_as_notes_section() {
        let p = project("Notas");
        let pid = p.id.clone();
        let docs = vec![doc(
            "a",
            &pid,
            None,
            "Cap",
            DocumentType::Chapter,
            Some(
                r#"<p>Alpha<sup data-footnote-id="x" data-footnote-content="Una nota">†</sup>.</p>"#,
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
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("class=\"notes\""));
        assert!(html.contains("Una nota"));
    }
}
