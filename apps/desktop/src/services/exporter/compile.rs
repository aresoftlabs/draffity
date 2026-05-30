//! Format-agnostic compile passes applied to the document set before any
//! renderer runs: drop research material (I-10), find&replace (K-02) and
//! front/back-matter reordering (K-03). Pure functions over `DocNode`, lifted
//! out of the export command so they're testable on their own (AUD-26).

use std::collections::{HashMap, HashSet};

use crate::domain::DocNode;

use super::config::FindReplaceRule;

/// Remove research documents (`is_research`) and every descendant of one.
/// A document is research if it carries the flag itself or any ancestor does.
pub fn strip_research(documents: Vec<DocNode>) -> Vec<DocNode> {
    let by_id: HashMap<&str, &DocNode> = documents.iter().map(|d| (d.id.as_str(), d)).collect();

    // Memoised "is this document inside a research subtree?" walk.
    let mut research: HashSet<String> = HashSet::new();
    for doc in &documents {
        let mut chain: Vec<&str> = Vec::new();
        let mut cursor: Option<&str> = Some(doc.id.as_str());
        let mut hit = false;
        while let Some(id) = cursor {
            if research.contains(id) {
                hit = true;
                break;
            }
            let Some(node) = by_id.get(id) else { break };
            chain.push(id);
            if node.is_research {
                hit = true;
                break;
            }
            cursor = node.parent_id.as_deref();
        }
        if hit {
            for id in chain {
                research.insert(id.to_string());
            }
        }
    }

    documents
        .into_iter()
        .filter(|d| !research.contains(&d.id))
        .collect()
}

/// Apply compile find&replace rules (K-02) to each document's content. Literal
/// (non-regex) replacement, in order. Export-only — storage is untouched.
pub fn apply_find_replace(documents: &mut [DocNode], rules: &[FindReplaceRule]) {
    let active: Vec<&FindReplaceRule> = rules.iter().filter(|r| !r.pattern.is_empty()).collect();
    if active.is_empty() {
        return;
    }
    for doc in documents.iter_mut() {
        if let Some(html) = doc.content.as_mut() {
            for rule in &active {
                if html.contains(&rule.pattern) {
                    *html = html.replace(&rule.pattern, &rule.replacement);
                }
            }
        }
    }
}

/// Reorder so front-matter top-level docs compile first and back-matter last
/// (K-03). Achieved by rewriting `position` of root docs into three bands;
/// renderers sort by position within each parent, so subtrees stay intact.
pub fn reorder_matter(documents: &mut [DocNode]) {
    const FRONT: i64 = -1_000_000;
    const BACK: i64 = 1_000_000;
    for doc in documents.iter_mut() {
        if doc.parent_id.is_some() {
            continue; // only reorder at the top level
        }
        if doc.is_front_matter {
            doc.position += FRONT;
        } else if doc.is_back_matter {
            doc.position += BACK;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DocumentStatus, DocumentType};

    fn doc(id: &str, parent: Option<&str>, is_research: bool) -> DocNode {
        DocNode {
            id: id.into(),
            project_id: "p".into(),
            parent_id: parent.map(|s| s.into()),
            title: id.into(),
            doc_type: DocumentType::Folder,
            content: None,
            content_json: None,
            synopsis: None,
            position: 0,
            status: DocumentStatus::Draft,
            tags: vec![],
            label_ids: vec![],
            metadata: std::collections::HashMap::new(),
            is_research,
            is_front_matter: false,
            is_back_matter: false,
            goal_words: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn rule(pattern: &str, replacement: &str) -> FindReplaceRule {
        FindReplaceRule {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    #[test]
    fn find_replace_rewrites_content_only_for_matching_rules() {
        let mut docs = vec![doc("a", None, false), doc("b", None, false)];
        docs[0].content = Some("<p>[SPOILER] hidden</p>".into());
        docs[1].content = Some("<p>clean</p>".into());
        apply_find_replace(&mut docs, &[rule("[SPOILER] ", ""), rule("", "x")]);
        assert_eq!(docs[0].content.as_deref(), Some("<p>hidden</p>"));
        // Empty-pattern rule is ignored; untouched doc stays as-is.
        assert_eq!(docs[1].content.as_deref(), Some("<p>clean</p>"));
    }

    #[test]
    fn reorder_matter_pushes_front_first_and_back_last() {
        let mut docs = vec![
            doc("body", None, false),
            doc("title", None, false),
            doc("appendix", None, false),
            doc("child", Some("body"), false),
        ];
        docs[0].position = 1;
        docs[1].position = 0;
        docs[1].is_front_matter = true;
        docs[2].position = 2;
        docs[2].is_back_matter = true;
        docs[3].position = 5; // a child — must NOT be reordered
        reorder_matter(&mut docs);
        assert!(docs[1].position < docs[0].position); // front before body
        assert!(docs[2].position > docs[0].position); // back after body
        assert_eq!(docs[3].position, 5); // child untouched
    }

    #[test]
    fn strips_research_root_and_descendants_keeps_the_rest() {
        // tree: ch1 (manuscript); research (flagged) → note → deep
        let docs = vec![
            doc("ch1", None, false),
            doc("research", None, true),
            doc("note", Some("research"), false),
            doc("deep", Some("note"), false),
        ];
        let kept: Vec<String> = strip_research(docs).into_iter().map(|d| d.id).collect();
        assert_eq!(kept, vec!["ch1".to_string()]);
    }

    #[test]
    fn keeps_everything_when_no_research() {
        let docs = vec![doc("a", None, false), doc("b", Some("a"), false)];
        assert_eq!(strip_research(docs).len(), 2);
    }
}
