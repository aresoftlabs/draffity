//! DOCX importer. A DOCX is a ZIP with `word/document.xml` carrying the
//! manuscript body in the OOXML wordprocessingml namespace. We open the
//! archive, read that one entry and walk the XML emitting:
//!
//!   - one `ImportNode` per heading paragraph, nested by level
//!   - HTML body content for non-heading paragraphs, accumulated under
//!     the most recent heading
//!
//! We only honour the subset of formatting the editor supports today:
//! bold, italic, underline, strikethrough and paragraph nesting up to H6.
//! Tables and footnotes are dropped silently — they're not lossless yet
//! and surfacing partial support would mislead the user about round-trip.

use std::io::{Cursor, Read};

use roxmltree::Document;
use zip::ZipArchive;

use crate::error::{AppError, AppResult};

use super::{ImportNode, ImportTree};

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

pub(super) fn import(bytes: &[u8], filename_hint: &str) -> AppResult<ImportTree> {
    let xml = read_document_xml(bytes)?;
    let dom =
        Document::parse(&xml).map_err(|e| AppError::Invariant(format!("invalid DOCX XML: {e}")))?;

    let sections = collect_paragraphs(&dom);
    let nodes = build_tree(&sections);
    let project_title = sections
        .iter()
        .find(|s| s.level.is_some_and(|l| l == 1))
        .map(|s| s.title_or_text())
        .unwrap_or_else(|| filename_hint.to_string());
    Ok(ImportTree {
        project_title,
        nodes,
    })
}

fn read_document_xml(bytes: &[u8]) -> AppResult<String> {
    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)
        .map_err(|e| AppError::Invariant(format!("DOCX is not a valid ZIP: {e}")))?;
    let mut entry = zip
        .by_name("word/document.xml")
        .map_err(|_| AppError::Invariant("DOCX missing word/document.xml entry".into()))?;
    let mut xml = String::new();
    entry.read_to_string(&mut xml)?;
    Ok(xml)
}

/// One paragraph captured from the DOCX. `level: Some(n)` marks it as a
/// heading; `None` means body content. We keep the raw inline HTML — the
/// tree builder concatenates body paragraphs into the preceding heading's
/// content_html.
struct ParaSection {
    level: Option<u8>,
    text_for_title: String,
    html: String,
}

impl ParaSection {
    fn title_or_text(&self) -> String {
        self.text_for_title.trim().to_string()
    }
}

fn collect_paragraphs(dom: &Document) -> Vec<ParaSection> {
    let mut out = Vec::new();
    for p in descendants_named(dom.root_element(), "p") {
        let level = heading_level(p);
        let mut text_buf = String::new();
        let mut html_buf = String::new();
        for run in p.children().filter(|n| is_named(n, "r")) {
            collect_run(run, &mut text_buf, &mut html_buf);
        }
        // Skip blank paragraphs: they pollute the body with empty `<p>`s
        // and inflate the chapter HTML for no gain.
        if text_buf.trim().is_empty() && level.is_none() {
            continue;
        }
        let html = if level.is_some() {
            // Heading paragraphs don't contribute body HTML — the title is
            // promoted to its own ImportNode.
            String::new()
        } else {
            format!("<p>{html_buf}</p>")
        };
        out.push(ParaSection {
            level,
            text_for_title: text_buf,
            html,
        });
    }
    out
}

fn heading_level(p: roxmltree::Node<'_, '_>) -> Option<u8> {
    let p_pr = p.children().find(|n| is_named(n, "pPr"))?;
    let style = p_pr.children().find(|n| is_named(n, "pStyle"))?;
    let val = style.attribute((W_NS, "val"))?;
    parse_heading_style(val)
}

/// Map Word style names like `Heading1`/`Ttulo1`/`Heading 1` to a 1-6
/// level. Returns `None` when the style isn't a heading at all.
fn parse_heading_style(val: &str) -> Option<u8> {
    let lower = val.to_ascii_lowercase().replace([' ', '_', '-'], "");
    let stripped = lower
        .strip_prefix("heading")
        .or_else(|| lower.strip_prefix("titulo"))
        .or_else(|| lower.strip_prefix("título"))
        .or_else(|| lower.strip_prefix("ttulo"))?;
    stripped.parse::<u8>().ok().filter(|n| (1..=6).contains(n))
}

fn collect_run(run: roxmltree::Node<'_, '_>, text_buf: &mut String, html_buf: &mut String) {
    let marks = inline_marks(run);
    let mut text = String::new();
    for child in run.children().filter(|n| is_named(n, "t")) {
        if let Some(t) = child.text() {
            text.push_str(t);
        }
    }
    if text.is_empty() {
        // Watch for `<w:br/>` — preserve as soft break in the HTML.
        if run.children().any(|n| is_named(&n, "br")) {
            html_buf.push_str("<br/>");
        }
        return;
    }
    text_buf.push_str(&text);
    let mut wrapped = escape_html(&text);
    if marks.code {
        wrapped = format!("<code>{wrapped}</code>");
    }
    if marks.strike {
        wrapped = format!("<s>{wrapped}</s>");
    }
    if marks.underline {
        wrapped = format!("<u>{wrapped}</u>");
    }
    if marks.italic {
        wrapped = format!("<em>{wrapped}</em>");
    }
    if marks.bold {
        wrapped = format!("<strong>{wrapped}</strong>");
    }
    html_buf.push_str(&wrapped);
}

#[derive(Default)]
struct RunMarks {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    code: bool,
}

fn inline_marks(run: roxmltree::Node<'_, '_>) -> RunMarks {
    let mut marks = RunMarks::default();
    let Some(r_pr) = run.children().find(|n| is_named(n, "rPr")) else {
        return marks;
    };
    for child in r_pr.children() {
        match local_name(child) {
            "b" if !explicit_false(child) => marks.bold = true,
            "i" if !explicit_false(child) => marks.italic = true,
            // `<w:u w:val="none">` actively disables underline; treat
            // anything else as "on".
            "u" if child.attribute((W_NS, "val")) != Some("none") => {
                marks.underline = true;
            }
            "strike" if !explicit_false(child) => marks.strike = true,
            "rStyle" if child.attribute((W_NS, "val")) == Some("Code") => marks.code = true,
            _ => {}
        }
    }
    marks
}

/// `<w:b w:val="false"/>` actively disables bold; treat the attribute as
/// "off" when val parses as a falsy boolean. Default presence means on.
fn explicit_false(node: roxmltree::Node<'_, '_>) -> bool {
    matches!(
        node.attribute((W_NS, "val")),
        Some("0" | "false" | "off") | Some("False")
    )
}

fn is_named(n: &roxmltree::Node<'_, '_>, local: &str) -> bool {
    n.is_element() && n.tag_name().name() == local && n.tag_name().namespace() == Some(W_NS)
}

fn local_name<'a>(n: roxmltree::Node<'a, '_>) -> &'a str {
    n.tag_name().name()
}

fn descendants_named<'a, 'd>(
    root: roxmltree::Node<'a, 'd>,
    local: &'a str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'd>> + 'a {
    root.descendants().filter(move |n| is_named(n, local))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn build_tree(sections: &[ParaSection]) -> Vec<ImportNode> {
    // Same hoist rule as the Markdown importer: if there is exactly one
    // H1 at the very top and it has children below it, the H1 lifts to
    // project_title and only its descendants become roots.
    let h1_count = sections.iter().filter(|s| s.level == Some(1)).count();
    let first_is_h1 = sections.first().and_then(|s| s.level) == Some(1);
    let first_has_children = sections
        .iter()
        .skip(1)
        .any(|s| s.level.is_some_and(|l| l > 1));
    let skip_first_h1 = h1_count == 1 && first_is_h1 && first_has_children;

    let mut roots: Vec<ImportNode> = Vec::new();
    let mut path: Vec<(u8, Vec<usize>)> = Vec::new();

    let mut iter = sections.iter().enumerate().peekable();
    while let Some((i, sec)) = iter.next() {
        match sec.level {
            Some(level) => {
                if skip_first_h1 && i == 0 {
                    continue;
                }
                let node = ImportNode {
                    title: sec.title_or_text(),
                    content_html: String::new(),
                    children: Vec::new(),
                };
                while path.last().map(|(l, _)| *l >= level).unwrap_or(false) {
                    path.pop();
                }
                if path.is_empty() {
                    roots.push(node);
                    path.push((level, vec![roots.len() - 1]));
                } else {
                    let parent_path = path.last().unwrap().1.clone();
                    let parent = follow_mut(&mut roots, &parent_path);
                    parent.children.push(node);
                    let mut new_path = parent_path;
                    new_path.push(parent.children.len() - 1);
                    path.push((level, new_path));
                }
            }
            None => {
                // Body paragraph — attach to the deepest open heading. If
                // none yet (file starts with body content), synthesize an
                // Intro root.
                if let Some((_, parent_path)) = path.last() {
                    let pp = parent_path.clone();
                    let parent = follow_mut(&mut roots, &pp);
                    parent.content_html.push_str(&sec.html);
                } else {
                    let mut intro = ImportNode {
                        title: "Intro".to_string(),
                        content_html: sec.html.clone(),
                        children: Vec::new(),
                    };
                    // Append further consecutive body paragraphs.
                    while let Some((_, next)) = iter.peek() {
                        if next.level.is_some() {
                            break;
                        }
                        intro.content_html.push_str(&next.html);
                        iter.next();
                    }
                    roots.push(intro);
                    path.push((1, vec![roots.len() - 1]));
                }
            }
        }
    }
    roots
}

fn follow_mut<'a>(roots: &'a mut [ImportNode], path: &[usize]) -> &'a mut ImportNode {
    let (first, rest) = path.split_first().expect("non-empty path");
    let mut node = &mut roots[*first];
    for &idx in rest {
        node = &mut node.children[idx];
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal DOCX ZIP in memory with a single `word/document.xml`
    /// entry. The OOXML spec also requires `[Content_Types].xml` and
    /// `_rels/.rels` for Word to open the file — but our importer only
    /// reads `word/document.xml`, so we can skip the boilerplate in tests.
    fn make_docx(document_xml: &str) -> Vec<u8> {
        use std::io::Write;
        use zip::write::FileOptions;
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let options: FileOptions<'_, ()> =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("word/document.xml", options).unwrap();
            zip.write_all(document_xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    const WRAPPER_OPEN: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>"#;
    const WRAPPER_CLOSE: &str = "</w:body></w:document>";

    fn doc(body: &str) -> Vec<u8> {
        let xml = format!("{WRAPPER_OPEN}{body}{WRAPPER_CLOSE}");
        make_docx(&xml)
    }

    #[test]
    fn extracts_heading_levels_and_nesting() {
        let docx = doc(r#"
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Acto 1</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Cap 1</w:t></w:r></w:p>
<w:p><w:r><w:t>Body of cap 1.</w:t></w:r></w:p>
"#);
        let tree = import(&docx, "x").unwrap();
        assert_eq!(tree.project_title, "Acto 1");
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].title, "Cap 1");
        assert!(tree.nodes[0].content_html.contains("Body of cap 1"));
    }

    #[test]
    fn promotes_bold_italic_and_underline_to_inline_marks() {
        let docx = doc(r#"
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Cap</w:t></w:r></w:p>
<w:p>
  <w:r><w:rPr><w:b/></w:rPr><w:t>bold</w:t></w:r>
  <w:r><w:t> </w:t></w:r>
  <w:r><w:rPr><w:i/></w:rPr><w:t>italic</w:t></w:r>
  <w:r><w:t> </w:t></w:r>
  <w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>under</w:t></w:r>
</w:p>
"#);
        let tree = import(&docx, "x").unwrap();
        let html = &tree.nodes[0].content_html;
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<u>under</u>"));
    }

    #[test]
    fn skips_blank_paragraphs() {
        let docx = doc(r#"
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Cap</w:t></w:r></w:p>
<w:p><w:r><w:t></w:t></w:r></w:p>
<w:p><w:r><w:t>Real body.</w:t></w:r></w:p>
"#);
        let tree = import(&docx, "x").unwrap();
        let html = &tree.nodes[0].content_html;
        assert!(html.contains("Real body"));
        // Single body paragraph only, no empty <p></p>.
        assert_eq!(html.matches("<p>").count(), 1);
    }

    #[test]
    fn falls_back_to_filename_when_no_heading_paragraph_exists() {
        let docx = doc("<w:p><w:r><w:t>Just text.</w:t></w:r></w:p>");
        let tree = import(&docx, "loose").unwrap();
        assert_eq!(tree.project_title, "loose");
        assert_eq!(tree.nodes.len(), 1);
        assert!(tree.nodes[0].content_html.contains("Just text"));
    }

    #[test]
    fn rejects_non_zip_payload() {
        let res = import(b"plain text not a zip", "x");
        assert!(matches!(res, Err(AppError::Invariant(_))));
    }

    #[test]
    fn handles_spanish_titulo_style_name() {
        // Spanish locale Word uses `Ttulo1` (after stripping the
        // accented character) as the style name — we match it too.
        let docx = doc(
            r#"<w:p><w:pPr><w:pStyle w:val="Ttulo1"/></w:pPr><w:r><w:t>Capitulo</w:t></w:r></w:p>
<w:p><w:r><w:t>Body</w:t></w:r></w:p>"#,
        );
        let tree = import(&docx, "x").unwrap();
        assert_eq!(tree.project_title, "Capitulo");
    }
}
