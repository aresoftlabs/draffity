//! Markdown export. Single-file Markdown with YAML frontmatter at the top.
//! Uses `html2md` to convert TipTap HTML content to Markdown.

use crate::domain::{DocNode, Project};
use crate::error::AppResult;

use super::config::ExportConfig;
use super::util::{flatten_in_order, yaml_quote};

pub fn render(
    project: &Project,
    documents: &[DocNode],
    _config: &ExportConfig,
) -> AppResult<Vec<u8>> {
    let mut out = String::new();

    // Frontmatter
    out.push_str("---\n");
    out.push_str(&format!("title: {}\n", yaml_quote(&project.title)));
    out.push_str(&format!(
        "templateId: {}\n",
        yaml_quote(&project.template_id)
    ));
    out.push_str(&format!("createdAt: {}\n", project.created_at));
    out.push_str(&format!("updatedAt: {}\n", project.updated_at));
    if let Some(meta) = &project.metadata {
        let metadata_inline = serde_json::to_string(meta)?;
        out.push_str(&format!("metadata: {metadata_inline}\n"));
    }
    out.push_str("---\n\n");

    // Project title
    out.push_str(&format!("# {}\n\n", project.title));

    // Document tree
    let ordered = flatten_in_order(documents);
    for (depth, doc) in ordered {
        // Document title heading: project is h1, top-level docs h2, sub h3, etc.
        // Cap at h6 — beyond that we keep text but stop adding hashes.
        let level = (depth + 2).min(6);
        out.push_str(&"#".repeat(level));
        out.push(' ');
        out.push_str(&doc.title);
        out.push_str("\n\n");

        if let Some(html) = &doc.content {
            if !html.trim().is_empty() {
                let md = html2md::parse_html(html);
                out.push_str(md.trim());
                out.push_str("\n\n");
            }
        }
    }

    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DocumentType;
    use crate::services::exporter::test_support::{doc, project};

    #[test]
    fn includes_frontmatter_and_titles() {
        let p = project("Mi novela");
        let pid = p.id.clone();
        let docs = vec![
            doc("a", &pid, None, "Acto 1", DocumentType::Folder, None, 0),
            doc(
                "b",
                &pid,
                Some("a"),
                "Capítulo 1",
                DocumentType::Chapter,
                Some("<p>Hola <strong>mundo</strong>.</p>"),
                0,
            ),
        ];
        let bytes = render(&p, &docs, &ExportConfig::default()).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.starts_with("---\n"));
        assert!(text.contains("title: \"Mi novela\""));
        assert!(text.contains("# Mi novela"));
        assert!(text.contains("## Acto 1"));
        assert!(text.contains("### Capítulo 1"));
        // html2md should produce a bold marker for <strong>.
        assert!(text.contains("**mundo**"));
    }

    #[test]
    fn empty_project_still_has_frontmatter() {
        let p = project("X");
        let bytes = render(&p, &[], &ExportConfig::default()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("---\n"));
        assert!(text.contains("# X"));
    }
}
