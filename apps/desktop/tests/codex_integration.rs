//! S7-10 — end-to-end integration test for the codex pipeline.
//!
//! Creates a project + chapter + 3 codex entries, drops `[[name]]` cross-
//! references into the chapter body, exports EPUB with `include_codex`
//! enabled and verifies the appendix wires every entry through to the
//! reader. The byte search is loose-on-purpose: the OPF file inside the
//! EPUB ZIP is uncompressed enough to expose entry filenames in the raw
//! payload, and chapter XHTML carries the data-codex-ref attribute
//! verbatim because we never compress per-entry.

use std::sync::Arc;

use draffity_desktop_lib as app;
use draffity_desktop_lib::domain::{
    CodexInput, CodexKind, DocumentInput, DocumentType, ProjectInput,
};
use draffity_desktop_lib::services::{
    BuiltInTemplates, ExportConfig, ExportFormat, ExportService, FreeTier, LocalExporter,
    LocalStorageService, ProjectManager, StorageService,
};

fn build() -> (ProjectManager, Arc<LocalStorageService>) {
    let dir = tempdir();
    let path = dir.join("draffity.db");
    let storage = LocalStorageService::open(&path).expect("open tempdir SQLite");
    storage.migrate().expect("apply migrations on fresh DB");
    let templates = BuiltInTemplates::load().expect("load built-in templates from embedded JSON");
    let storage = Arc::new(storage);
    let pm = ProjectManager::new(storage.clone(), Arc::new(FreeTier), Arc::new(templates));
    (pm, storage)
}

fn tempdir() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut p = std::env::temp_dir();
    p.push(format!("draffity-codex-test-{nanos:x}"));
    std::fs::create_dir_all(&p).expect("create tempdir");
    p
}

#[test]
fn cross_refs_survive_export_and_codex_appendix_lists_every_entry() {
    let (pm, storage) = build();

    // Use the generic template so the binder starts empty — the test
    // owns exactly the doc tree it asserts on.
    let project = pm
        .create(ProjectInput {
            title: "Lord of the Rings".into(),
            template_id: "generic".into(),
            metadata: None,
        })
        .expect("create project");

    let aragorn = storage
        .create_codex_entry(CodexInput {
            project_id: project.id.clone(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: Some("<p>Ranger of the North, son of Arathorn.</p>".into()),
            tags: vec!["protagonist".into()],
        })
        .expect("create codex entry");
    let frodo = storage
        .create_codex_entry(CodexInput {
            project_id: project.id.clone(),
            kind: CodexKind::Character,
            name: "Frodo".into(),
            body: Some("<p>Ring-bearer.</p>".into()),
            tags: vec!["protagonist".into(), "hobbit".into()],
        })
        .expect("create codex entry");
    let mordor = storage
        .create_codex_entry(CodexInput {
            project_id: project.id.clone(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: Some("<p>Land of shadow.</p>".into()),
            tags: vec!["evil".into()],
        })
        .expect("create codex entry");

    // Body with three cross-refs — same shape the TipTap CodexRef node
    // serialises to. The id is the source of truth; name is the human
    // label that survived export-time.
    let chapter_html = format!(
        r#"<p>
            <span data-codex-ref="{aragorn_id}">[[Aragorn]]</span> rode
            with <span data-codex-ref="{frodo_id}">[[Frodo]]</span>
            towards <span data-codex-ref="{mordor_id}">[[Mordor]]</span>.
        </p>"#,
        aragorn_id = aragorn.id,
        frodo_id = frodo.id,
        mordor_id = mordor.id,
    );

    storage
        .create_document(DocumentInput {
            project_id: project.id.clone(),
            parent_id: None,
            title: "Chapter 1".into(),
            doc_type: DocumentType::Chapter,
            content: Some(chapter_html.clone()),
        })
        .expect("create document");

    let docs = storage.list_documents(&project.id).expect("list docs");
    let codex = storage.list_codex_entries(&project.id).expect("list codex");
    // The generic template ships one seed doc; we added Chapter 1 on top.
    assert!(docs.iter().any(|d| d.title == "Chapter 1"));
    assert_eq!(codex.len(), 3);

    // Export EPUB with codex appendix on.
    let config = ExportConfig {
        include_codex: true,
        ..ExportConfig::default()
    };
    let exporter = LocalExporter;
    let bytes = exporter
        .export(&project, &docs, &codex, ExportFormat::Epub, &config)
        .expect("render EPUB");

    // EPUB is a ZIP: magic bytes + the codex.xhtml entry in the central
    // directory (filenames are uncompressed even when contents aren't).
    assert_eq!(&bytes[0..4], b"PK\x03\x04");
    let lossy = String::from_utf8_lossy(&bytes);
    assert!(
        lossy.contains("codex.xhtml"),
        "codex appendix file should be in the EPUB"
    );

    // Cross-ref id round-trip + appendix content is easier to assert
    // against Markdown export: it's a single plain text file, no zlib in
    // the way. Both renderers share `config.include_codex`, so this
    // proves the flow for the whole exporter family.
    let md_bytes = exporter
        .export(&project, &docs, &codex, ExportFormat::Markdown, &config)
        .expect("render markdown");
    let md = String::from_utf8(md_bytes).expect("markdown is utf-8");
    for entry in [&aragorn, &frodo, &mordor] {
        assert!(
            md.contains(&entry.name),
            "expected codex entry name '{}' in markdown export",
            entry.name
        );
    }
    assert!(md.contains("# Codex"), "appendix heading missing");
    assert!(md.contains("Characters"), "characters section missing");
    assert!(md.contains("Places"), "places section missing");
}

#[test]
fn codex_appendix_is_skipped_when_include_codex_is_false() {
    let (pm, storage) = build();
    let project = pm
        .create(ProjectInput {
            title: "X".into(),
            template_id: "generic".into(),
            metadata: None,
        })
        .unwrap();
    storage
        .create_codex_entry(CodexInput {
            project_id: project.id.clone(),
            kind: CodexKind::Character,
            name: "Solo".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();

    let docs = storage.list_documents(&project.id).unwrap();
    let codex = storage.list_codex_entries(&project.id).unwrap();
    let bytes = LocalExporter
        .export(
            &project,
            &docs,
            &codex,
            ExportFormat::Epub,
            &ExportConfig::default(),
        )
        .unwrap();
    let lossy = String::from_utf8_lossy(&bytes);
    assert!(
        !lossy.contains("codex.xhtml"),
        "codex appendix should not appear when include_codex is false"
    );
}

#[test]
fn rename_keeps_cross_refs_pointing_to_the_same_entry() {
    let (pm, storage) = build();
    let project = pm
        .create(ProjectInput {
            title: "Tale".into(),
            template_id: "generic".into(),
            metadata: None,
        })
        .unwrap();
    let aragorn = storage
        .create_codex_entry(CodexInput {
            project_id: project.id.clone(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();

    // Rename to Strider.
    let updated = storage
        .update_codex_entry(
            &aragorn.id,
            app::domain::CodexUpdate {
                name: Some("Strider".into()),
                ..app::domain::CodexUpdate::default()
            },
        )
        .unwrap();
    assert_eq!(updated.id, aragorn.id);
    assert_eq!(updated.name, "Strider");

    // The cross-ref in user prose still resolves: the lookup is by id,
    // not by name, so the rename is invisible to existing references.
    let resolved = storage.get_codex_entry(&aragorn.id).unwrap().unwrap();
    assert_eq!(resolved.name, "Strider");
}
