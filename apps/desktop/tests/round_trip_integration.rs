//! C-03 — exporters ↔ importers round-trip integration.
//!
//! Builds a canonical project tree, exports it to a format, imports the
//! resulting bytes back into an `ImportTree`, then asserts the normalised
//! shape survives. The expectations are intentionally loose on
//! presentation (exact whitespace, marker characters) and tight on
//! identity-preserving content: titles, paragraph text, heading nesting,
//! footnote bodies. Inline marks survive DOCX but not Markdown (html2md
//! converts `<strong>` → `**bold**` which the Markdown importer renders
//! back into `<strong>` — the text content matches, the surrounding
//! markup is the renderer's choice).
//!
//! Tables and images are explicitly out of scope: the importers don't
//! parse them yet, so a round-trip would always lose them. The tests for
//! those land in Sprint C follow-ups once the importer grows table /
//! image support.

use draffity_desktop_lib::domain::{
    new_id, now_ms, DocNode, DocumentStatus, DocumentType, Project, ProjectStatus,
};
use draffity_desktop_lib::services::exporter::MediaBundle;
use draffity_desktop_lib::services::importer::{ImportFormat, ImportService, LocalImporter};
use draffity_desktop_lib::services::ExportConfig;
use draffity_desktop_lib::services::{ExportFormat, ExportService, LocalExporter};

fn project(title: &str) -> Project {
    let now = now_ms();
    Project {
        id: new_id(),
        title: title.into(),
        template_id: "generic".into(),
        status: ProjectStatus::Active,
        metadata: None,
        goal_words: None,
        created_at: now,
        updated_at: now,
    }
}

fn doc(
    id: &str,
    project_id: &str,
    parent_id: Option<&str>,
    title: &str,
    doc_type: DocumentType,
    content: Option<&str>,
    position: i64,
) -> DocNode {
    let now = now_ms();
    DocNode {
        id: id.into(),
        project_id: project_id.into(),
        parent_id: parent_id.map(|s| s.into()),
        title: title.into(),
        doc_type,
        content: content.map(|s| s.into()),
        content_json: None,
        synopsis: None,
        position,
        status: DocumentStatus::Draft,
        tags: Vec::new(),
        label_ids: Vec::new(),
        metadata: std::collections::HashMap::new(),
        is_research: false,
        goal_words: None,
        created_at: now,
        updated_at: now,
    }
}

/// Strip HTML tags + collapse whitespace; mirrors what the diff util
/// does on the frontend. Two HTML strings whose visible text matches
/// will normalise to the same value.
fn textify(html: &str) -> String {
    let no_tags: String = {
        let mut out = String::with_capacity(html.len());
        let mut in_tag = false;
        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => {
                    in_tag = false;
                    out.push(' ');
                }
                _ if !in_tag => out.push(ch),
                _ => {}
            }
        }
        out
    };
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn markdown_round_trip_preserves_titles_and_body_text() {
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
        doc(
            "c",
            &pid,
            Some("a"),
            "Capítulo 2",
            DocumentType::Chapter,
            Some("<p>Otro <em>capítulo</em>.</p>"),
            1,
        ),
    ];

    let bytes = LocalExporter
        .export(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            ExportFormat::Markdown,
            &ExportConfig::default(),
        )
        .expect("markdown render");

    let tree = LocalImporter
        .import(ImportFormat::Markdown, &bytes, "fallback")
        .expect("markdown import");

    assert_eq!(tree.project_title, "Mi novela");

    // After hoisting the H1 "Mi novela" to project_title, the root is
    // "Acto 1" with two H3-derived chapters under it.
    assert_eq!(tree.nodes.len(), 1);
    let acto = &tree.nodes[0];
    assert_eq!(acto.title, "Acto 1");
    assert_eq!(acto.children.len(), 2);
    let cap1 = &acto.children[0];
    let cap2 = &acto.children[1];
    assert_eq!(cap1.title, "Capítulo 1");
    assert_eq!(cap2.title, "Capítulo 2");
    assert!(textify(&cap1.content_html).contains("Hola mundo"));
    assert!(textify(&cap2.content_html).contains("Otro capítulo"));
}

#[test]
fn markdown_round_trip_preserves_footnotes() {
    let p = project("Notas");
    let pid = p.id.clone();
    let docs = vec![doc(
        "a",
        &pid,
        None,
        "Cap",
        DocumentType::Chapter,
        Some(
            r#"<p>Alfa<sup data-footnote-id="x" data-footnote-content="Una aclaración">†</sup> y omega.</p>"#,
        ),
        0,
    )];

    let bytes = LocalExporter
        .export(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            ExportFormat::Markdown,
            &ExportConfig::default(),
        )
        .unwrap();
    let tree = LocalImporter
        .import(ImportFormat::Markdown, &bytes, "x")
        .unwrap();

    // The body should make it back with the footnote body inlined.
    let body = &tree
        .nodes
        .first()
        .map(|n| n.content_html.as_str())
        .unwrap_or("");
    assert!(
        body.contains("data-footnote-content=\"Una aclaración\""),
        "footnote body lost in round-trip: {body}"
    );
}

#[test]
fn docx_round_trip_preserves_titles_and_body_text() {
    let p = project("Manuscrito");
    let pid = p.id.clone();
    let docs = vec![doc(
        "a",
        &pid,
        None,
        "Capítulo 1",
        DocumentType::Chapter,
        Some("<p>El comienzo.</p>"),
        0,
    )];

    let bytes = LocalExporter
        .export(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            ExportFormat::Docx,
            // Skip TOC / title page — they fire chapter promotion that
            // the importer's heading-level rules don't (yet) align with.
            // Round-trip identity is preserved on the content stream
            // itself, which is what matters here.
            &ExportConfig {
                include_toc: false,
                include_title_page: false,
                ..ExportConfig::default()
            },
        )
        .unwrap();

    let tree = LocalImporter
        .import(ImportFormat::Docx, &bytes, "fallback")
        .unwrap();

    // We don't assert tree.project_title equals the source title — the
    // DOCX exporter's title-page omission means the first heading in the
    // file is "Capítulo 1", which is fine as a project title.
    let body_text: String = tree
        .nodes
        .iter()
        .map(|n| textify(&n.content_html))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        body_text.contains("El comienzo"),
        "docx body lost in round-trip: {body_text}"
    );
}

#[test]
fn docx_round_trip_preserves_inline_marks() {
    let p = project("Marcas");
    let pid = p.id.clone();
    let docs = vec![doc(
        "a",
        &pid,
        None,
        "Cap",
        DocumentType::Chapter,
        Some("<p><strong>bold</strong> <em>italic</em> <u>under</u></p>"),
        0,
    )];

    let bytes = LocalExporter
        .export(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            ExportFormat::Docx,
            &ExportConfig {
                include_toc: false,
                include_title_page: false,
                ..ExportConfig::default()
            },
        )
        .unwrap();
    let tree = LocalImporter
        .import(ImportFormat::Docx, &bytes, "x")
        .unwrap();

    let body = tree
        .nodes
        .iter()
        .map(|n| n.content_html.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(body.contains("<strong>bold</strong>"), "bold lost: {body}");
    assert!(body.contains("<em>italic</em>"), "italic lost: {body}");
    assert!(body.contains("<u>under</u>"), "underline lost: {body}");
}

#[test]
fn markdown_round_trip_preserves_heading_hierarchy() {
    let p = project("Estructura");
    let pid = p.id.clone();
    let docs = vec![
        doc("a", &pid, None, "Parte 1", DocumentType::Folder, None, 0),
        doc(
            "b",
            &pid,
            Some("a"),
            "Capítulo A",
            DocumentType::Chapter,
            Some("<p>A.</p>"),
            0,
        ),
        doc(
            "c",
            &pid,
            Some("b"),
            "Escena 1",
            DocumentType::Scene,
            Some("<p>Escena uno.</p>"),
            0,
        ),
    ];

    let bytes = LocalExporter
        .export(
            &p,
            &docs,
            &[],
            &MediaBundle::new(),
            ExportFormat::Markdown,
            &ExportConfig::default(),
        )
        .unwrap();

    let tree = LocalImporter
        .import(ImportFormat::Markdown, &bytes, "x")
        .unwrap();

    assert_eq!(tree.project_title, "Estructura");
    // Parte 1 (folder, H2) → Capítulo A (H3) → Escena 1 (H4).
    let parte = &tree.nodes[0];
    assert_eq!(parte.title, "Parte 1");
    let cap = &parte.children[0];
    assert_eq!(cap.title, "Capítulo A");
    let escena = &cap.children[0];
    assert_eq!(escena.title, "Escena 1");
    assert!(textify(&escena.content_html).contains("Escena uno"));
}
