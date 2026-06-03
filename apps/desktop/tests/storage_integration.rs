//! End-to-end integration tests for the storage + project manager pipeline.
//! Uses an on-disk temporary SQLite to exercise the same code path as
//! production (migrations, foreign keys, partial unique index).

use std::sync::Arc;

use draffity_desktop_lib as app;

use app::services::{
    BuiltInTemplates, LocalProjectManager, LocalStorageService, ProjectManagerService,
    StorageService,
};

fn build() -> Box<dyn ProjectManagerService> {
    let dir = tempdir();
    let path = dir.join("draffity.db");
    let storage = LocalStorageService::open(&path).expect("open tempdir SQLite");
    storage.migrate().expect("apply migrations on fresh DB");
    let templates = BuiltInTemplates::load().expect("load built-in templates from embedded JSON");
    Box::new(LocalProjectManager::new(
        Arc::new(storage),
        Arc::new(templates),
    ))
}

fn tempdir() -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("draffity-test-{}", uuid_like()));
    std::fs::create_dir_all(&p).expect("create tempdir for integration test");
    p
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[test]
fn full_lifecycle_active_then_switch_then_reactivate() {
    let pm = build();

    let novela = pm
        .create(app::domain::ProjectInput {
            title: "Mi novela".into(),
            template_id: "novela-tres-actos".into(),
            metadata: None,
        })
        .unwrap();
    assert_eq!(novela.status, app::domain::ProjectStatus::Active);

    let paper = pm
        .create(app::domain::ProjectInput {
            title: "Paper IMRaD".into(),
            template_id: "paper-imrad".into(),
            metadata: None,
        })
        .unwrap();
    assert_eq!(paper.status, app::domain::ProjectStatus::Active);

    // Single active project invariant: only one active.
    let active = pm.active().unwrap().unwrap();
    assert_eq!(active.id, paper.id);
    let stored_novela = pm.get(&novela.id).unwrap().unwrap();
    assert_eq!(stored_novela.status, app::domain::ProjectStatus::Archived);

    // Re-activate the novel.
    pm.activate(&novela.id).unwrap();
    let active = pm.active().unwrap().unwrap();
    assert_eq!(active.id, novela.id);
    let stored_paper = pm.get(&paper.id).unwrap().unwrap();
    assert_eq!(stored_paper.status, app::domain::ProjectStatus::Archived);
}
