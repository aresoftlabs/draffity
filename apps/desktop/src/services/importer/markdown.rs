//! Markdown importer. Splits a single Markdown file into a tree of
//! documents using `#`-headings as the nesting hierarchy:
//!
//!   H1                    → top-level node (or sole project title)
//!     H2                  → child of the preceding H1
//!       H3                → child of the preceding H2
//!         ...
//!
//! Body text in between headings is captured into the *preceding*
//! heading's content. Footnote definitions (`[^N]: …`) are gathered and
//! re-inlined as TipTap `<sup data-footnote-id="…" data-footnote-content="…">`
//! so round-trip with the exporter survives.

use std::collections::HashMap;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::domain::new_id;
use crate::error::{AppError, AppResult};

use super::{ImportFormat, ImportNode, ImportService, ImportTree};

/// Standalone Markdown importer — useful when a caller only needs the
/// Markdown path and doesn't want the format dispatch in `LocalImporter`.
/// Implementing `ImportService` for it keeps the trait surface uniform.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalMarkdownImporter;

impl ImportService for LocalMarkdownImporter {
    fn supported_formats(&self) -> Vec<ImportFormat> {
        vec![ImportFormat::Markdown]
    }

    fn import(
        &self,
        format: ImportFormat,
        bytes: &[u8],
        filename_hint: &str,
    ) -> AppResult<ImportTree> {
        match format {
            ImportFormat::Markdown => import(bytes, filename_hint),
            ImportFormat::Docx => Err(AppError::Unsupported(
                "Markdown importer cannot read DOCX".into(),
            )),
        }
    }
}

pub(super) fn import(bytes: &[u8], filename_hint: &str) -> AppResult<ImportTree> {
    let text = String::from_utf8_lossy(bytes);
    let (stripped_frontmatter, frontmatter_title) = peel_frontmatter(&text);
    // Capture footnote bodies *without* stripping them — pulldown-cmark
    // needs the definitions in place to actually recognise `[^id]` as a
    // FootnoteReference event. The renderer filters out the definition
    // events after the fact so they don't bleed into chapter bodies.
    // Trim leading whitespace so a blank line right after the
    // frontmatter close (or before the first heading) doesn't promote
    // itself to a synthetic "Intro" section.
    let body = stripped_frontmatter.trim_start_matches(['\n', '\r']);
    let footnotes = scan_footnote_defs(body);

    let sections = split_by_headings(body);
    let nodes = build_tree(&sections, &footnotes);

    // Title priority: YAML frontmatter `title:` > first explicit H1 >
    // filename. Synthetic sections (body-before-heading fallback) don't
    // promote themselves, otherwise no-heading files would get titled
    // "Intro".
    let project_title = frontmatter_title
        .or_else(|| {
            sections
                .iter()
                .find(|s| s.level == 1 && !s.synthetic)
                .map(|s| s.title.clone())
        })
        .unwrap_or_else(|| filename_hint.to_string());

    Ok(ImportTree {
        project_title,
        nodes,
    })
}

/// Strip a leading `---\n…\n---\n` YAML block (if present) and pull the
/// `title:` field out of it. We don't run a full YAML parser — the
/// exporter only emits a flat key:value map and the import-side `title`
/// is the only field we care about today.
fn peel_frontmatter(text: &str) -> (&str, Option<String>) {
    let trimmed = text.trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---\n") else {
        return (text, None);
    };
    let Some(close_pos) = rest.find("\n---\n") else {
        return (text, None);
    };
    let block = &rest[..close_pos];
    let after = &rest[close_pos + "\n---\n".len()..];
    let title = block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let rest = trimmed.strip_prefix("title:")?;
        Some(unquote(rest.trim()))
    });
    (after, title)
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Scan for `[^id]: body` lines and return the `{id -> body}` map. The
/// definitions are left in place in the markdown so pulldown-cmark
/// recognises the matching `[^id]` references when it parses them.
/// Multiline (indented continuation) bodies aren't supported — the
/// exporter doesn't emit them either.
fn scan_footnote_defs(body: &str) -> HashMap<String, String> {
    let mut defs = HashMap::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("[^") {
            if let Some(close) = rest.find("]: ") {
                let id = &rest[..close];
                let content = rest[close + "]: ".len()..].trim().to_string();
                defs.insert(id.to_string(), content);
            }
        }
    }
    defs
}

#[derive(Debug, Clone)]
struct Section {
    level: u8,
    title: String,
    body_md: String,
    /// True for the synthetic "Intro" section we generate when a file
    /// has body content before its first heading. Synthetic sections
    /// don't promote themselves to project_title and don't trigger the
    /// "single H1 ↑ project_title" hoist.
    synthetic: bool,
}

/// Walk the raw Markdown text scanning for ATX headings (`# … ######`)
/// and split into sections. Each section keeps its raw Markdown body so
/// the renderer can convert just that slice to HTML — that's cheaper
/// than walking the full Pulldown event stream and lets us reuse the
/// crate's standard HTML renderer per-section.
fn split_by_headings(body: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;
    for line in body.lines() {
        if let Some((level, title)) = parse_heading(line) {
            if let Some(s) = current.take() {
                sections.push(s);
            }
            current = Some(Section {
                level,
                title,
                body_md: String::new(),
                synthetic: false,
            });
        } else if let Some(s) = current.as_mut() {
            s.body_md.push_str(line);
            s.body_md.push('\n');
        } else {
            // Content before the first heading lands in a synthetic
            // "Intro" section so we don't lose it.
            current = Some(Section {
                level: 1,
                title: "Intro".to_string(),
                body_md: format!("{line}\n"),
                synthetic: true,
            });
        }
    }
    if let Some(s) = current.take() {
        sections.push(s);
    }
    sections
}

fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.starts_with(' ') {
        return None;
    }
    Some((hashes as u8, rest.trim().to_string()))
}

/// Build the import tree by walking sections in document order and
/// nesting each one under the most recent section of a strictly smaller
/// level. The project title (H1 when it matches) is dropped from the
/// tree to avoid duplicating it as both project name and root document.
fn build_tree(sections: &[Section], footnotes: &HashMap<String, String>) -> Vec<ImportNode> {
    // Skip the first H1 only when it serves purely as a project title:
    // - it must be the first section (not synthetic),
    // - it must be the only H1 in the document, AND
    // - it must have nested children (H2+) — otherwise dropping it
    //   would lose the only chapter the file contains.
    let h1_count = sections.iter().filter(|s| s.level == 1).count();
    let first_is_real_h1 = sections
        .first()
        .map(|s| s.level == 1 && !s.synthetic)
        .unwrap_or(false);
    let first_has_children = sections.get(1).map(|s| s.level > 1).unwrap_or(false);
    let skip_first_h1 = h1_count == 1 && first_is_real_h1 && first_has_children;

    let mut roots: Vec<ImportNode> = Vec::new();
    // Stack of (level, indices-path-from-roots). We walk down to insert.
    let mut path: Vec<(u8, Vec<usize>)> = Vec::new();

    for (i, sec) in sections.iter().enumerate() {
        if skip_first_h1 && i == 0 {
            continue;
        }
        let node = ImportNode {
            title: sec.title.clone(),
            content_html: render_markdown_html(&sec.body_md, footnotes),
            children: Vec::new(),
        };
        // Pop levels >= sec.level so we land at the right depth.
        while path.last().map(|(l, _)| *l >= sec.level).unwrap_or(false) {
            path.pop();
        }
        if path.is_empty() {
            roots.push(node);
            path.push((sec.level, vec![roots.len() - 1]));
        } else {
            let parent_path = path.last().unwrap().1.clone();
            let parent = follow_mut(&mut roots, &parent_path);
            parent.children.push(node);
            let mut new_path = parent_path;
            new_path.push(parent.children.len() - 1);
            path.push((sec.level, new_path));
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

/// Render a Markdown body (the slice between two headings) to HTML and
/// re-inline footnote references as TipTap `<sup>` nodes so the editor
/// recognises them on save.
fn render_markdown_html(md: &str, footnotes: &HashMap<String, String>) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_FOOTNOTES);

    // Strip nested headings so they don't reappear inside the body —
    // they'd duplicate what `split_by_headings` already promoted into
    // their own sections. FootnoteDefinition blocks are also filtered:
    // we replay the bodies inline via the `<sup>` rewrite, so the raw
    // definition list at the bottom of the chapter would be dead weight.
    let mut in_fn_def = false;
    let parser = Parser::new_ext(md, opts).filter(move |event| match event {
        Event::Start(Tag::Heading { .. }) | Event::End(TagEnd::Heading(_)) => false,
        Event::Start(Tag::FootnoteDefinition(_)) => {
            in_fn_def = true;
            false
        }
        Event::End(TagEnd::FootnoteDefinition) => {
            in_fn_def = false;
            false
        }
        _ if in_fn_def => false,
        _ => true,
    });

    // Translate footnote references emitted by pulldown-cmark
    // (`<sup class="footnote-reference"><a href="#fr-X-1">…</a></sup>`)
    // into our editor format. Pulldown surfaces footnotes via
    // `Event::FootnoteReference("id")`, so map there for an exact rewrite.
    let mapped = parser.map(|event| match event {
        Event::FootnoteReference(id) => {
            let content = footnotes.get(id.as_ref()).cloned().unwrap_or_default();
            let fn_id = new_id();
            Event::Html(
                format!(
                    "<sup class=\"footnote-ref\" data-footnote-id=\"{fn_id}\" data-footnote-content=\"{}\">†</sup>",
                    escape_attr(&content)
                )
                .into(),
            )
        }
        other => other,
    });

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, mapped);
    html.trim().to_string()
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulls_title_from_frontmatter_then_strips_block() {
        let md = b"---\ntitle: \"Mi novela\"\nauthor: Anon\n---\n\n# Cap 1\n\nHola.";
        let tree = import(md, "fallback").unwrap();
        assert_eq!(tree.project_title, "Mi novela");
        // The frontmatter block must not leak into the first section body.
        let first = &tree.nodes[0];
        assert!(!first.content_html.contains("author"));
    }

    #[test]
    fn nests_h2_under_preceding_h1() {
        let md = b"# Acto 1\n\nIntro acto.\n\n## Capitulo 1\n\nCuerpo capitulo.\n";
        let tree = import(md, "x").unwrap();
        // Project title becomes "Acto 1" and skip_first_h1 drops the H1,
        // so the only root is the H2.
        assert_eq!(tree.project_title, "Acto 1");
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].title, "Capitulo 1");
        assert!(tree.nodes[0].content_html.contains("Cuerpo capitulo"));
    }

    #[test]
    fn keeps_all_h1s_when_there_are_several() {
        let md = b"# Cap 1\n\nA.\n\n# Cap 2\n\nB.\n";
        let tree = import(md, "anthology").unwrap();
        assert_eq!(tree.project_title, "Cap 1");
        // Two top-level docs (no skip — multi-H1 = anthology).
        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.nodes[1].title, "Cap 2");
    }

    #[test]
    fn renders_footnote_refs_as_tiptap_sup() {
        let md = b"# Cap\n\nHola[^a] mundo.\n\n[^a]: una nota\n";
        let tree = import(md, "x").unwrap();
        let html = &tree.nodes.first().map(|n| &n.content_html);
        // Since the only H1 is dropped into project_title, the body
        // landed in an Intro section synthesised before nothing — so we
        // assert against the first node.
        let body = html.expect("at least one node");
        assert!(
            body.contains("data-footnote-content=\"una nota\""),
            "body was: {body}"
        );
    }

    #[test]
    fn falls_back_to_filename_when_no_title_anywhere() {
        let md = b"Just a paragraph without any heading.\n";
        let tree = import(md, "draft").unwrap();
        assert_eq!(tree.project_title, "draft");
        // Body lands in a synthetic Intro section.
        assert_eq!(tree.nodes.len(), 1);
        assert!(tree.nodes[0].content_html.contains("Just a paragraph"));
    }

    #[test]
    fn returns_at_least_one_doc_for_well_formed_file() {
        let md = b"# Top\n\n## Sub\n\nbody";
        let tree = import(md, "x").unwrap();
        // Top becomes project_title, Sub is the single root.
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].title, "Sub");
    }
}
