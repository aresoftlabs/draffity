//! Shared helpers for the export renderers.

use std::collections::HashMap;

use crate::domain::DocNode;

/// Preorder traversal of the document tree. Returns each node paired with its
/// depth (0 = root). Documents are ordered by `position` within each parent.
pub fn flatten_in_order(documents: &[DocNode]) -> Vec<(usize, &DocNode)> {
    let mut by_parent: HashMap<Option<&str>, Vec<&DocNode>> = HashMap::new();
    for d in documents {
        by_parent.entry(d.parent_id.as_deref()).or_default().push(d);
    }
    for v in by_parent.values_mut() {
        v.sort_by_key(|d| d.position);
    }

    let mut out = Vec::with_capacity(documents.len());
    walk(&by_parent, None, 0, &mut out);
    out
}

fn walk<'a>(
    by_parent: &HashMap<Option<&'a str>, Vec<&'a DocNode>>,
    parent: Option<&'a str>,
    depth: usize,
    out: &mut Vec<(usize, &'a DocNode)>,
) {
    let Some(children) = by_parent.get(&parent) else {
        return;
    };
    for child in children {
        out.push((depth, *child));
        walk(by_parent, Some(child.id.as_str()), depth + 1, out);
    }
}

/// Escape characters that would break XML/XHTML payloads. Does not handle
/// attribute escaping (we never emit user content into attributes).
pub fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape `:` and `\n` and quote a YAML string value for frontmatter use.
/// Keeps things simple — we always emit double-quoted strings.
pub fn yaml_quote(input: &str) -> String {
    let mut s = String::from("\"");
    for ch in input.chars() {
        match ch {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            '\n' => s.push_str("\\n"),
            '\r' => {}
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04X}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::doc;

    #[test]
    fn flatten_preserves_preorder_with_depth() {
        let docs = vec![
            doc("root1", "p", None, "Acto 1", DocumentType::Folder, None, 0),
            doc("root2", "p", None, "Acto 2", DocumentType::Folder, None, 1),
            doc(
                "child2",
                "p",
                Some("root1"),
                "Cap 2",
                DocumentType::Chapter,
                None,
                1,
            ),
            doc(
                "child1",
                "p",
                Some("root1"),
                "Cap 1",
                DocumentType::Chapter,
                None,
                0,
            ),
        ];
        let out = flatten_in_order(&docs);
        let titles: Vec<_> = out.iter().map(|(d, n)| (*d, n.title.as_str())).collect();
        assert_eq!(
            titles,
            vec![(0, "Acto 1"), (1, "Cap 1"), (1, "Cap 2"), (0, "Acto 2"),]
        );
    }

    #[test]
    fn xml_escape_handles_special_chars() {
        assert_eq!(
            xml_escape("a & <b> \"c\""),
            "a &amp; &lt;b&gt; &quot;c&quot;"
        );
    }

    #[test]
    fn yaml_quote_escapes_quotes_and_newlines() {
        assert_eq!(yaml_quote("hi \"x\"\nline"), "\"hi \\\"x\\\"\\nline\"");
    }
}
