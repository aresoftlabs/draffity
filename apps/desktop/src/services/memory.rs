//! Project memory for the AI assistant (F-03). **Engram-aligned, lexical tier.**
//!
//! A single retriever assembles the context the inline-AI (Épica F) and the
//! validators (Épica G) get, from what the project *already* holds — no new
//! model, no embeddings, no infra:
//!   - **semantic memory** = Codex entries mentioned in the working window,
//!   - **episodic memory** = prior passages retrieved via the existing FTS5,
//!   - **synopses** = document + project synopsis.
//!
//! The embeddings upgrade (ONNX + sqlite-vec) plugs in later behind this same
//! trait without touching consumers. See `backlog-v4.md` → "Arquitectura de
//! memoria del asistente".

use std::sync::Arc;

use crate::error::AppResult;
use crate::services::storage::StorageService;
use crate::services::token_counter::estimate_tokens;

/// What to build context for. `window_text` is the plain text near the
/// cursor/selection that anchors relevance (mentions + search terms).
#[derive(Debug, Clone)]
pub struct MemoryRequest {
    pub project_id: String,
    pub doc_id: Option<String>,
    pub window_text: String,
    /// Soft cap; episodic excerpts are dropped first when the rendered
    /// context would exceed it.
    pub token_budget: u32,
}

impl MemoryRequest {
    /// Default budget for the assembled memory block, leaving room in a
    /// typical context window for the manuscript window + system prompt.
    pub const DEFAULT_BUDGET: u32 = 4_000;
}

/// Retrieved, typed memory. `render()` turns it into the prompt block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemoryContext {
    pub project_synopsis: Option<String>,
    pub doc_synopsis: Option<String>,
    pub codex_snippets: Vec<String>,
    pub episodic_excerpts: Vec<String>,
}

impl MemoryContext {
    pub fn is_empty(&self) -> bool {
        self.project_synopsis.is_none()
            && self.doc_synopsis.is_none()
            && self.codex_snippets.is_empty()
            && self.episodic_excerpts.is_empty()
    }

    /// Render to a single prompt block. Empty context renders to an empty
    /// string so callers can omit the section entirely.
    pub fn render(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut out = String::from("== Memoria del proyecto ==\n");
        if let Some(s) = &self.project_synopsis {
            out.push_str(&format!("Sinopsis del proyecto: {s}\n"));
        }
        if let Some(s) = &self.doc_synopsis {
            out.push_str(&format!("Sinopsis del documento actual: {s}\n"));
        }
        if !self.codex_snippets.is_empty() {
            out.push_str("\nEntradas del codex relevantes:\n");
            for c in &self.codex_snippets {
                out.push_str(&format!("- {c}\n"));
            }
        }
        if !self.episodic_excerpts.is_empty() {
            out.push_str("\nPasajes anteriores relevantes:\n");
            for e in &self.episodic_excerpts {
                out.push_str(&format!("- {e}\n"));
            }
        }
        out
    }
}

pub trait ProjectMemoryService: Send + Sync {
    fn build_context(&self, req: &MemoryRequest) -> AppResult<MemoryContext>;
}

/// Lexical retriever over Codex + FTS5 + synopses. No embeddings.
pub struct LexicalProjectMemory {
    storage: Arc<dyn StorageService>,
}

/// Max episodic excerpts pulled, and per-excerpt char cap (keeps the block
/// bounded before budget trimming even kicks in).
const MAX_EPISODIC: usize = 5;
const EXCERPT_CHARS: usize = 300;

impl LexicalProjectMemory {
    pub fn new(storage: Arc<dyn StorageService>) -> Self {
        Self { storage }
    }
}

impl ProjectMemoryService for LexicalProjectMemory {
    fn build_context(&self, req: &MemoryRequest) -> AppResult<MemoryContext> {
        let window_lc = req.window_text.to_lowercase();
        let mut ctx = MemoryContext::default();

        // --- semantic memory: codex entries mentioned in the window ---
        let entries = self.storage.list_codex_entries(&req.project_id)?;
        let mut mentioned_names: Vec<String> = Vec::new();
        for e in &entries {
            let name_lc = e.name.to_lowercase();
            if name_lc.is_empty() || !window_lc.contains(&name_lc) {
                continue;
            }
            mentioned_names.push(e.name.clone());
            let body = e.body.as_deref().unwrap_or("—").trim();
            let tags = if e.tags.is_empty() {
                String::new()
            } else {
                format!(" [tags: {}]", e.tags.join(", "))
            };
            ctx.codex_snippets.push(format!(
                "{} ({}): {}{}",
                e.name,
                e.kind.as_str(),
                body,
                tags
            ));
        }

        // --- episodic memory: prior passages referencing those entities ---
        // One FTS query per mentioned name; take the first hit from a *different*
        // document than the one being edited, so we surface context, not the
        // sentence the user is staring at. Dedup by document.
        let mut seen_docs: std::collections::HashSet<String> = std::collections::HashSet::new();
        for name in &mentioned_names {
            if ctx.episodic_excerpts.len() >= MAX_EPISODIC {
                break;
            }
            let hits = self.storage.search_documents(&req.project_id, name)?;
            for hit in hits {
                if req.doc_id.as_deref() == Some(hit.document_id.as_str()) {
                    continue;
                }
                if !seen_docs.insert(hit.document_id.clone()) {
                    continue;
                }
                let excerpt = clean_excerpt(&hit.excerpt);
                if !excerpt.is_empty() {
                    ctx.episodic_excerpts
                        .push(format!("«{}» — {}", hit.title, excerpt));
                }
                break;
            }
        }

        // --- synopses ---
        if let Some(doc_id) = &req.doc_id {
            if let Some(doc) = self.storage.get_document(doc_id)? {
                ctx.doc_synopsis = doc.synopsis.filter(|s| !s.trim().is_empty());
            }
        }
        if let Some(project) = self.storage.get_project(&req.project_id)? {
            ctx.project_synopsis = project
                .metadata
                .as_ref()
                .and_then(|m| m.get("synopsis"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.trim().is_empty());
        }

        trim_to_budget(&mut ctx, req.token_budget);
        Ok(ctx)
    }
}

/// Strip the FTS `<mark>` wrappers and clamp length for prompt embedding.
fn clean_excerpt(raw: &str) -> String {
    let stripped = raw.replace("<mark>", "").replace("</mark>", "");
    let trimmed = stripped.trim();
    if trimmed.chars().count() > EXCERPT_CHARS {
        let truncated: String = trimmed.chars().take(EXCERPT_CHARS).collect();
        format!("{truncated}…")
    } else {
        trimmed.to_string()
    }
}

/// Drop episodic excerpts (cheapest to lose) from the end until the rendered
/// block fits the budget. Codex + synopses are kept — they're the high-signal
/// core and small.
fn trim_to_budget(ctx: &mut MemoryContext, budget: u32) {
    while !ctx.episodic_excerpts.is_empty() && estimate_tokens(&ctx.render()) > budget {
        ctx.episodic_excerpts.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CodexInput, CodexKind, DocumentInput, DocumentType, ProjectInput};
    use crate::services::storage::{LocalStorageService, StorageService};

    fn fresh() -> Arc<dyn StorageService> {
        let s = LocalStorageService::open_in_memory().unwrap();
        s.migrate().unwrap();
        Arc::new(s)
    }

    fn seed_project(s: &Arc<dyn StorageService>) -> String {
        s.create_project(ProjectInput {
            title: "Novela".into(),
            template_id: "x".into(),
            metadata: Some(serde_json::json!({ "synopsis": "Una saga de anillos." })),
        })
        .unwrap()
        .id
    }

    #[test]
    fn mentioned_codex_entry_is_included_others_are_not() {
        let s = fresh();
        let pid = seed_project(&s);
        s.create_codex_entry(CodexInput {
            project_id: pid.clone(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: Some("Montaraz del Norte, ojos grises.".into()),
            tags: vec!["héroe".into()],
        })
        .unwrap();
        s.create_codex_entry(CodexInput {
            project_id: pid.clone(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: Some("Tierra oscura.".into()),
            tags: vec![],
        })
        .unwrap();

        let mem = LexicalProjectMemory::new(s.clone());
        let ctx = mem
            .build_context(&MemoryRequest {
                project_id: pid,
                doc_id: None,
                window_text: "Aragorn desenvainó la espada.".into(),
                token_budget: MemoryRequest::DEFAULT_BUDGET,
            })
            .unwrap();

        assert_eq!(ctx.codex_snippets.len(), 1, "only the mentioned entry");
        assert!(ctx.codex_snippets[0].contains("Aragorn"));
        assert!(ctx.codex_snippets[0].contains("character"));
        assert!(ctx.codex_snippets.iter().all(|c| !c.contains("Mordor")));
    }

    #[test]
    fn synopses_are_picked_up() {
        let s = fresh();
        let pid = seed_project(&s);
        let doc = s
            .create_document(DocumentInput {
                project_id: pid.clone(),
                parent_id: None,
                title: "Capítulo 1".into(),
                doc_type: DocumentType::Chapter,
                content: Some("<p>texto</p>".into()),
            })
            .unwrap();
        s.set_document_synopsis(&doc.id, Some("El héroe parte de casa."))
            .unwrap();

        let mem = LexicalProjectMemory::new(s.clone());
        let ctx = mem
            .build_context(&MemoryRequest {
                project_id: pid,
                doc_id: Some(doc.id),
                window_text: "nada relevante".into(),
                token_budget: MemoryRequest::DEFAULT_BUDGET,
            })
            .unwrap();

        assert_eq!(
            ctx.project_synopsis.as_deref(),
            Some("Una saga de anillos.")
        );
        assert_eq!(ctx.doc_synopsis.as_deref(), Some("El héroe parte de casa."));
        assert!(ctx.render().contains("Sinopsis del proyecto"));
    }

    #[test]
    fn episodic_excerpt_comes_from_a_different_document() {
        let s = fresh();
        let pid = seed_project(&s);
        // Prior chapter that mentions the character.
        s.create_document(DocumentInput {
            project_id: pid.clone(),
            parent_id: None,
            title: "Capítulo previo".into(),
            doc_type: DocumentType::Chapter,
            content: Some("<p>Aragorn cruzó el río al amanecer.</p>".into()),
        })
        .unwrap();
        // Current chapter being edited.
        let current = s
            .create_document(DocumentInput {
                project_id: pid.clone(),
                parent_id: None,
                title: "Capítulo actual".into(),
                doc_type: DocumentType::Chapter,
                content: Some("<p>Aragorn aquí.</p>".into()),
            })
            .unwrap();
        s.create_codex_entry(CodexInput {
            project_id: pid.clone(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();

        let mem = LexicalProjectMemory::new(s.clone());
        let ctx = mem
            .build_context(&MemoryRequest {
                project_id: pid,
                doc_id: Some(current.id.clone()),
                window_text: "Aragorn".into(),
                token_budget: MemoryRequest::DEFAULT_BUDGET,
            })
            .unwrap();

        assert!(
            ctx.episodic_excerpts
                .iter()
                .any(|e| e.contains("Capítulo previo")),
            "should surface the prior chapter, got {:?}",
            ctx.episodic_excerpts
        );
        assert!(
            ctx.episodic_excerpts
                .iter()
                .all(|e| !e.contains("Capítulo actual")),
            "must not echo the document being edited"
        );
    }

    #[test]
    fn tiny_budget_drops_episodic_but_keeps_codex() {
        let s = fresh();
        let pid = seed_project(&s);
        s.create_document(DocumentInput {
            project_id: pid.clone(),
            parent_id: None,
            title: "Previo".into(),
            doc_type: DocumentType::Chapter,
            content: Some("<p>Aragorn cabalgó largo y tendido por la pradera.</p>".into()),
        })
        .unwrap();
        s.create_codex_entry(CodexInput {
            project_id: pid.clone(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: Some("Montaraz.".into()),
            tags: vec![],
        })
        .unwrap();

        let mem = LexicalProjectMemory::new(s.clone());
        let ctx = mem
            .build_context(&MemoryRequest {
                project_id: pid,
                doc_id: None,
                window_text: "Aragorn".into(),
                token_budget: 5, // absurdly small
            })
            .unwrap();

        assert!(
            ctx.episodic_excerpts.is_empty(),
            "episodic dropped under budget"
        );
        assert_eq!(ctx.codex_snippets.len(), 1, "codex kept");
    }

    #[test]
    fn empty_context_renders_empty_string() {
        let ctx = MemoryContext::default();
        assert!(ctx.is_empty());
        assert_eq!(ctx.render(), "");
    }
}
