//! Generates a deterministic fixture EPUB at the path passed as `argv[1]`.
//!
//! Used by CI (S1-06) to validate the export pipeline against `epubcheck`.
//! Builds an in-memory project with a handful of chapters, runs the
//! `LocalExporter`, and writes the bytes to disk.

use std::env;

use draffity_desktop_lib::{
    domain::{DocumentInput, DocumentType, ProjectInput},
    services::{
        ExportConfig, ExportFormat, ExportService, LocalExporter, LocalStorageService,
        StorageService,
    },
};

fn main() {
    let out = env::args()
        .nth(1)
        .expect("usage: cargo run --example fixture_epub -- <out.epub>");

    let storage =
        LocalStorageService::open_in_memory().expect("in-memory SQLite should always open");
    storage.migrate().expect("apply migrations");

    let project = storage
        .create_project(ProjectInput {
            title: "Fixture EPUB".into(),
            template_id: "novela-tres-actos".into(),
            metadata: None,
        })
        .expect("create project");

    for (i, body) in [
        "<p>Capítulo 1: el viaje empieza.</p>",
        "<p>Capítulo 2: el desafío.</p>",
        "<p>Capítulo 3: el desenlace.</p>",
    ]
    .iter()
    .enumerate()
    {
        storage
            .create_document(DocumentInput {
                project_id: project.id.clone(),
                parent_id: None,
                title: format!("Capítulo {}", i + 1),
                doc_type: DocumentType::Chapter,
                content: Some((*body).into()),
            })
            .expect("create document");
    }

    let docs = storage.list_documents(&project.id).expect("list documents");
    let bytes = LocalExporter
        .export(
            &project,
            &docs,
            &[],
            ExportFormat::Epub,
            &ExportConfig::default(),
        )
        .expect("render EPUB");

    std::fs::write(&out, &bytes).expect("write output file");
    println!("wrote {} bytes to {out}", bytes.len());
}
