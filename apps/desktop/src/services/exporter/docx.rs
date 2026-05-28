//! DOCX export. We parse the TipTap HTML with `scraper` and emit a stream of
//! paragraphs/headings/runs into a `docx-rs` document. List numbering is kept
//! simple (text prefix) — this is enough for an MVP-grade manuscript export.

use docx_rs::{AlignmentType, Docx, Paragraph, Run, RunFonts};
use scraper::{ElementRef, Html, Node};

use crate::domain::{DocNode, Project};
use crate::error::AppResult;

use super::config::ExportConfig;
use super::util::flatten_in_order;

pub fn render(
    project: &Project,
    documents: &[DocNode],
    _config: &ExportConfig,
) -> AppResult<Vec<u8>> {
    let mut docx = Docx::new();

    // Project title (H1, centered)
    docx = docx.add_paragraph(
        Paragraph::new()
            .style("Heading1")
            .align(AlignmentType::Center)
            .add_run(Run::new().add_text(project.title.clone()).size(48).bold()),
    );

    let ordered = flatten_in_order(documents);
    for (depth, doc) in ordered {
        let title_level = (depth + 2).min(6);
        // Document title heading.
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

/// Convert a TipTap HTML fragment to a sequence of `Paragraph`s.
fn render_html_blocks(html: &str) -> Vec<Paragraph> {
    let parsed = Html::parse_fragment(html);
    let mut out = Vec::new();
    for child in parsed.root_element().child_elements() {
        process_block(child, &mut out, BlockCtx::default());
    }
    out
}

#[derive(Default, Clone, Copy)]
struct BlockCtx {
    in_blockquote: bool,
}

#[derive(Clone, Copy)]
enum ListKind {
    Bullet,
    Ordered,
}

fn process_block(el: ElementRef<'_>, out: &mut Vec<Paragraph>, ctx: BlockCtx) {
    let name = el.value().name();
    match name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level: usize = name[1..].parse().unwrap_or(2);
            // Embedded headings nest under document title heading: shift by +2
            // (project=1, doc=2..6, embedded continues from there but cap at 6).
            let style = format!("Heading{}", (level + 2).min(6));
            let mut p = Paragraph::new().style(&style);
            for r in collect_runs(el) {
                p = p.add_run(r);
            }
            out.push(p);
        }
        "p" => {
            let mut p = Paragraph::new();
            if ctx.in_blockquote {
                p = p.style("Quote");
            }
            for r in collect_runs(el) {
                p = p.add_run(r);
            }
            out.push(p);
        }
        "blockquote" => {
            let _ = ctx; // ctx kept for future nesting flags
            let new_ctx = BlockCtx {
                in_blockquote: true,
            };
            for child in el.child_elements() {
                process_block(child, out, new_ctx);
            }
        }
        "ul" | "ol" => {
            let kind = if name == "ul" {
                ListKind::Bullet
            } else {
                ListKind::Ordered
            };
            for (idx, li) in
                (1usize..).zip(el.child_elements().filter(|e| e.value().name() == "li"))
            {
                let prefix = match kind {
                    ListKind::Bullet => "• ".to_string(),
                    ListKind::Ordered => format!("{idx}. "),
                };
                let mut runs = vec![Run::new().add_text(prefix)];
                runs.extend(collect_runs(li));
                let mut p = Paragraph::new();
                for r in runs {
                    p = p.add_run(r);
                }
                out.push(p);
            }
        }
        "hr" => {
            out.push(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().add_text("⸻")),
            );
        }
        "pre" => {
            // Code block — collapse to monospace text in a single paragraph.
            let text = el.text().collect::<String>();
            out.push(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(text)
                        .fonts(RunFonts::new().ascii("Consolas")),
                ),
            );
        }
        "br" => {
            out.push(Paragraph::new());
        }
        _ => {
            // Fallback: treat as a paragraph of plain inline content.
            let runs = collect_runs(el);
            if !runs.is_empty() {
                let mut p = Paragraph::new();
                for r in runs {
                    p = p.add_run(r);
                }
                out.push(p);
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct InlineMarks {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    code: bool,
}

impl InlineMarks {
    fn merged_with(&self, name: &str) -> Self {
        let mut m = *self;
        match name {
            "strong" | "b" => m.bold = true,
            "em" | "i" => m.italic = true,
            "u" => m.underline = true,
            "s" | "del" | "strike" => m.strike = true,
            "code" => m.code = true,
            _ => {}
        }
        m
    }

    fn apply(&self, mut r: Run) -> Run {
        if self.bold {
            r = r.bold();
        }
        if self.italic {
            r = r.italic();
        }
        if self.underline {
            r = r.underline("single");
        }
        if self.strike {
            r = r.strike();
        }
        if self.code {
            r = r.fonts(RunFonts::new().ascii("Consolas"));
        }
        r
    }
}

/// Walk `el`'s descendants, producing `Run`s with the appropriate inline
/// formatting. Inline-level only: block elements inside a paragraph are
/// flattened to their text.
fn collect_runs(el: ElementRef<'_>) -> Vec<Run> {
    let mut runs = Vec::new();
    for descendant in el.children() {
        push_runs(descendant, InlineMarks::default(), &mut runs);
    }
    runs
}

fn push_runs(node: ego_tree::NodeRef<'_, Node>, marks: InlineMarks, runs: &mut Vec<Run>) {
    match node.value() {
        Node::Text(t) => {
            let s = t.text.to_string();
            if !s.is_empty() {
                runs.push(marks.apply(Run::new().add_text(s)));
            }
        }
        Node::Element(e) => {
            let name = e.name();
            if name == "br" {
                runs.push(Run::new().add_break(docx_rs::BreakType::TextWrapping));
                return;
            }
            let next_marks = marks.merged_with(name);
            for child in node.children() {
                push_runs(child, next_marks, runs);
            }
        }
        _ => {}
    }
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
        let bytes = render(&p, &docs, &ExportConfig::default()).unwrap();
        // ZIP local file header signature
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        // Should contain at least the standard word/document.xml entry name.
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("word/document.xml"));
    }

    #[test]
    fn empty_project_still_produces_zip() {
        let p = project("X");
        let bytes = render(&p, &[], &ExportConfig::default()).unwrap();
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
    }
}
