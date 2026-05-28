//! DOCX export helpers: HTML→Paragraph rendering pipeline + Codex appendix
//! builder. Pulled out of `docx.rs` to keep `render()` as a thin orchestrator
//! and to fit the per-file size budget (CLAUDE.md §1).
//!
//! The `pub(super)` surface is the minimum that `docx.rs::render` and
//! `docx.rs::append_codex` need to call.

use docx_rs::{AlignmentType, BreakType, Docx, Paragraph, Run, RunFonts};
use scraper::{ElementRef, Html, Node};

use crate::domain::{CodexEntry, CodexKind};

/// Convert a TipTap HTML fragment to a sequence of `Paragraph`s. Used by the
/// chapter body pipeline and by the Codex appendix.
pub(super) fn render_html_blocks(html: &str) -> Vec<Paragraph> {
    let parsed = Html::parse_fragment(html);
    let mut out = Vec::new();
    for child in parsed.root_element().child_elements() {
        process_block(child, &mut out, BlockCtx::default());
    }
    out
}

/// Append a Codex appendix to the doc: page break, "Codex" heading and
/// then sections grouped by kind. Each entry is rendered as a Heading3
/// plus body HTML reusing the same `render_html_blocks` pipeline the doc
/// content uses, so formatting stays consistent.
pub(super) fn append_codex(mut docx: Docx, codex: &[CodexEntry]) -> Docx {
    docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)));
    docx = docx.add_paragraph(
        Paragraph::new()
            .style("Heading1")
            .add_run(Run::new().add_text("Codex").bold().size(48)),
    );

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
        docx = docx.add_paragraph(
            Paragraph::new()
                .style("Heading2")
                .add_run(Run::new().add_text(codex_section_name(kind)).bold()),
        );
        for e in entries {
            docx = append_codex_entry(docx, e);
        }
    }
    docx
}

fn append_codex_entry(mut docx: Docx, e: &CodexEntry) -> Docx {
    docx = docx.add_paragraph(
        Paragraph::new()
            .style("Heading3")
            .add_run(Run::new().add_text(e.name.clone()).bold()),
    );
    if !e.tags.is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(e.tags.join(", ")).italic()),
        );
    }
    if let Some(body) = &e.body {
        if !body.trim().is_empty() {
            for paragraph in render_html_blocks(body) {
                docx = docx.add_paragraph(paragraph);
            }
        }
    }
    docx
}

fn codex_section_name(kind: CodexKind) -> &'static str {
    match kind {
        CodexKind::Character => "Characters",
        CodexKind::Place => "Places",
        CodexKind::Object => "Objects",
        CodexKind::Note => "Notes",
    }
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

/// Dispatch a single block-level HTML element to the right paragraph builder.
/// Each branch is small enough to keep the function under the 50-line budget;
/// list and heading helpers live below.
fn process_block(el: ElementRef<'_>, out: &mut Vec<Paragraph>, ctx: BlockCtx) {
    let name = el.value().name();
    match name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => out.push(heading_paragraph(name, el)),
        "p" => out.push(paragraph_with_ctx(el, ctx)),
        "blockquote" => {
            let next = BlockCtx {
                in_blockquote: true,
            };
            for child in el.child_elements() {
                process_block(child, out, next);
            }
        }
        "ul" => render_list(el, ListKind::Bullet, out),
        "ol" => render_list(el, ListKind::Ordered, out),
        "hr" => out.push(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text("⸻")),
        ),
        "pre" => out.push(code_block_paragraph(el)),
        "br" => out.push(Paragraph::new()),
        _ => {
            // Fallback: treat as a paragraph of plain inline content.
            let runs = collect_runs(el);
            if !runs.is_empty() {
                out.push(runs_into_paragraph(runs));
            }
        }
    }
}

fn heading_paragraph(name: &str, el: ElementRef<'_>) -> Paragraph {
    let level: usize = name[1..].parse().unwrap_or(2);
    // Embedded headings nest under document title heading: shift by +2
    // (project=1, doc=2..6, embedded continues from there but cap at 6).
    let style = format!("Heading{}", (level + 2).min(6));
    let mut p = Paragraph::new().style(&style);
    for r in collect_runs(el) {
        p = p.add_run(r);
    }
    p
}

fn paragraph_with_ctx(el: ElementRef<'_>, ctx: BlockCtx) -> Paragraph {
    let mut p = Paragraph::new();
    if ctx.in_blockquote {
        p = p.style("Quote");
    }
    for r in collect_runs(el) {
        p = p.add_run(r);
    }
    p
}

fn render_list(el: ElementRef<'_>, kind: ListKind, out: &mut Vec<Paragraph>) {
    for (idx, li) in (1usize..).zip(el.child_elements().filter(|e| e.value().name() == "li")) {
        let prefix = match kind {
            ListKind::Bullet => "• ".to_string(),
            ListKind::Ordered => format!("{idx}. "),
        };
        let mut runs = vec![Run::new().add_text(prefix)];
        runs.extend(collect_runs(li));
        out.push(runs_into_paragraph(runs));
    }
}

fn code_block_paragraph(el: ElementRef<'_>) -> Paragraph {
    let text = el.text().collect::<String>();
    Paragraph::new().add_run(
        Run::new()
            .add_text(text)
            .fonts(RunFonts::new().ascii("Consolas")),
    )
}

fn runs_into_paragraph(runs: Vec<Run>) -> Paragraph {
    let mut p = Paragraph::new();
    for r in runs {
        p = p.add_run(r);
    }
    p
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
