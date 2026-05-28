//! Footnote pre-pass shared by Markdown and EPUB renderers. The TipTap
//! Footnote node persists as `<sup data-footnote-id="X" data-footnote-content="Y">…</sup>`
//! — the body lives inline as an attribute so the document is
//! self-contained.
//!
//! At export time each chapter's HTML is walked sequentially and every
//! marker is rewritten to a format-specific placeholder. The collected
//! `Footnote` list keeps insertion order so renderers can emit a numbered
//! "Notes" section at the end of the chapter / book.

/// One footnote ready to be rendered into the target format. `number` is
/// 1-based within the chapter; `content` is the raw text the user typed
/// (no HTML escaping applied — renderers handle that).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footnote {
    pub number: u32,
    pub content: String,
}

/// Replace every `<sup data-footnote-id="…" data-footnote-content="…">…</sup>`
/// in `html` with the string returned by `replace_with(number)` and return
/// the rewritten HTML alongside the ordered footnote list.
///
/// The scan is intentionally lexical — no full HTML parse — so it can run
/// before format-specific renderers and stay fast for large chapters.
/// Malformed or unterminated `<sup` openers are left as-is so the reader
/// can spot them in the output.
pub fn collect_footnotes<F>(html: &str, mut replace_with: F) -> (String, Vec<Footnote>)
where
    F: FnMut(u32) -> String,
{
    let mut out = String::with_capacity(html.len());
    let mut notes = Vec::new();
    let mut cursor = 0usize;
    let mut counter: u32 = 0;

    while let Some(rel_start) = html[cursor..].find("<sup") {
        let start = cursor + rel_start;
        out.push_str(&html[cursor..start]);
        // Find the tag end and the matching `</sup>`.
        let Some(tag_close_rel) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return (out, notes);
        };
        let tag_close = start + tag_close_rel + 1;
        let Some(end_rel) = html[tag_close..].find("</sup>") else {
            out.push_str(&html[start..]);
            return (out, notes);
        };
        let end = tag_close + end_rel + "</sup>".len();
        let tag = &html[start..tag_close];

        if let Some(content) = extract_attr(tag, "data-footnote-content") {
            counter += 1;
            notes.push(Footnote {
                number: counter,
                content,
            });
            out.push_str(&replace_with(counter));
        } else {
            // Not a footnote sup (could be superscript styling). Pass through.
            out.push_str(&html[start..end]);
        }
        cursor = end;
    }
    out.push_str(&html[cursor..]);
    (out, notes)
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(decode_html_entities(&rest[..end]))
}

/// Reverse the minimal entity-escape TipTap applies to attribute values.
/// We only handle the five XML predefined entities; that's enough for what
/// the editor produces.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_footnotes_in_document_order() {
        let html = r#"<p>Alpha<sup data-footnote-id="a" data-footnote-content="First note">†</sup> and beta<sup data-footnote-id="b" data-footnote-content="Second note">†</sup>.</p>"#;
        let (rewritten, notes) = collect_footnotes(html, |n| format!("[^{n}]"));
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].number, 1);
        assert_eq!(notes[0].content, "First note");
        assert_eq!(notes[1].number, 2);
        assert_eq!(notes[1].content, "Second note");
        assert!(rewritten.contains("Alpha[^1]"));
        assert!(rewritten.contains("beta[^2]"));
    }

    #[test]
    fn preserves_unrelated_sup_tags() {
        // A plain `<sup>` for superscript styling has no data attributes —
        // we leave it alone so the renderer can format it normally.
        let html = "<p>x<sup>2</sup> + y<sup>2</sup></p>";
        let (out, notes) = collect_footnotes(html, |n| format!("[^{n}]"));
        assert!(notes.is_empty());
        assert_eq!(out, html);
    }

    #[test]
    fn decodes_html_entities_in_attribute() {
        let html = r#"<p>A<sup data-footnote-id="a" data-footnote-content="X &amp; Y &lt; Z">†</sup>.</p>"#;
        let (_rewritten, notes) = collect_footnotes(html, |n| format!("[^{n}]"));
        assert_eq!(notes[0].content, "X & Y < Z");
    }

    #[test]
    fn handles_unterminated_sup_gracefully() {
        let html = "<p>Tail<sup data-footnote-content=\"foo\">";
        let (out, notes) = collect_footnotes(html, |n| format!("[^{n}]"));
        assert!(notes.is_empty());
        assert_eq!(out, html);
    }

    #[test]
    fn empty_input_returns_empty_results() {
        let (out, notes) = collect_footnotes("", |n| format!("[^{n}]"));
        assert!(out.is_empty());
        assert!(notes.is_empty());
    }
}
