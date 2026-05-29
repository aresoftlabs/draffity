//! Type-generator entry point for D-01. Walks the Rust domain types
//! annotated with `#[derive(specta::Type)]` and emits a TypeScript file
//! the frontend consumes as the source of truth for those types. CI
//! runs this binary and asserts the generated file matches what is
//! committed — any drift fails the build, so adding a new field on the
//! Rust side without re-running the generator surfaces immediately.
//!
//! Scope today: the simple enums that don't reach into `serde_json::Value`
//! or generic `Option<Vec<…>>` patterns specta's TypeScript exporter
//! still has rough edges around. The manual `index.ts` in
//! `packages/shared-types` keeps the rest until they migrate one by one.
//! Each new annotated type gets added to the `register_mut::<…>()` chain
//! below.

use std::path::PathBuf;

use draffity_desktop_lib::domain::{CodexKind, DocumentStatus, DocumentType, ProjectStatus};
use specta::Types;
use specta_typescript::Typescript;

fn main() {
    let mut types = Types::default();
    types.register_mut::<ProjectStatus>();
    types.register_mut::<DocumentType>();
    types.register_mut::<DocumentStatus>();
    types.register_mut::<CodexKind>();

    let out_path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default: write next to the manual shared-types index so a
            // local `cargo run --bin gen-types` produces the file CI checks
            // for drift.
            PathBuf::from("../../packages/shared-types/src/generated.ts")
        });

    let exporter = Typescript::default().header(BANNER);
    let rendered = exporter
        .export(&types, specta_serde::Format)
        .expect("specta TypeScript export should succeed for annotated types");
    std::fs::write(&out_path, rendered).expect("write generated TypeScript file");
    println!(
        "wrote {} types to {}",
        types.into_unsorted_iter().count(),
        out_path.display()
    );
}

const BANNER: &str = r#"// @generated — DO NOT EDIT.
// Produced by `cargo run --bin gen-types` from the desktop crate.
// Source: apps/desktop/src/bin/gen_types.rs (D-01).
"#;
