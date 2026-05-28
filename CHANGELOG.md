# Changelog

All notable changes to Draffity are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha] — 2026-05-08

First public alpha. Free MVP, premium-ready architecture.

### Added — Foundations

- Tauri 2 + Vue 3 + TypeScript + Vite scaffolding in a pnpm monorepo (`apps/desktop`, `apps/ui`, `packages/templates`, `packages/shared-types`).
- PrimeVue 4 + Tailwind CSS 3 + Pinia + Vue Router + vue-i18n (ES + EN).
- ESLint flat config + Prettier + clippy + rustfmt + husky + commitlint.
- GitHub Actions CI (lint + test + build matrix Windows + Linux).

### Added — Domain & Storage

- Pure domain entities: `Project`, `DocNode`, `Snapshot`, `Template`, `WritingStats`.
- SQLite v1 schema (`001_init.sql`) with **unique partial index `idx_projects_one_active`** enforcing the free-tier "1 active project" rule at the database level.
- `StorageService` trait + `LocalStorageService` (rusqlite, WAL, foreign keys ON).
- `TierService` + `FreeTier` driving `capabilities.rs` (single source of truth for feature gates).
- `ProjectManager` orchestrating the active/archive lifecycle with capability-aware behaviour.
- Premium-ready stubs: `AIService`, `CloudSyncService`, `ASRService`, `ExportService`.
- Atomic project + template instantiation (`create_project_atomic`) inside a single SQLite transaction.

### Added — Editor & Project UI

- TipTap editor wrapper with `StarterKit` + `Underline` + `Placeholder` + `CharacterCount`.
- `EditorToolbar` (H1/H2/H3, bold, italic, underline, strike, lists, blockquote, hr, undo/redo).
- 3-pane layout (`<Splitter>`): Binder (PrimeVue Tree, icons per `DocumentType`) · Editor · Inspector.
- Auto-save with 500ms debounce + `SaveIndicator` (idle / saving / saved / error).
- Read-only mode for archived projects (TipTap `editable=false` + banner + binder hides "create").
- Switching active project shows confirmation that the current one will be archived.
- Pinia stores (`project`, `document`, `ui`) with `localStorage` persistence for UI state.
- Composables: `useAutoSave`, `useIpcError`, `useShortcuts`, `useCapability`, `useEditorSettings`.
- Keyboard shortcuts: `Ctrl+S` (flush save), `Ctrl+N` (new chapter).

### Added — Templates Engine

- 4 built-in templates embedded via `include_str!`: `generic`, `novela-tres-actos`, `paper-imrad`, `manga-shonen`.
- Versioned schema (v1) with `kind`, `tier`, `locale`, recursive `structure`, dynamic `metadataFields`.
- `NewProjectWizard` with 3 steps: template + preview tree → metadata form (string/text/number/date) → confirmation.
- IPC: `list_templates`, `get_template`.

### Added — Export, Stats & Settings

- `LocalExporter` strategy with sub-modules for Markdown, DOCX (`docx-rs`), EPUB (`epub-builder`).
- HTML → Markdown via `html2md`; HTML → DOCX runs via `scraper` (headings, bold/italic/underline/strike/code, lists, blockquote, hr).
- EPUB output: one XHTML chapter per document with metadata.
- Snapshots: manual versioning with optional label, restore with auto-snapshot of the pre-restore state.
- Writing streak (current + longest) backed by `chrono` + `settings` table; auto-recorded on `update_document`.
- Settings UI: editor font (serif/sans/mono), auto-save interval slider (200–3000 ms), writing-stats panel.
- Export dialog with native save dialog (`@tauri-apps/plugin-dialog`).

### Added — Quality & Release

- Crash logging: `tracing-appender` daily-rotated file under `<app_data>/logs/`, mirrored to stderr.
- Panic hook captures `std::panic::set_hook` payload + location to log.
- Onboarding (3 slides) shown on first launch.
- `release.yml` workflow with `tauri-action` for tag-driven MSI/AppImage releases.
- `docs/USER-GUIDE.md` (ES) and `docs/RELEASE-CHECKLIST.md`.

### Architecture invariants

- **Premium is additive.** Adding premium implementations of any service trait must not modify domain, commands, or existing service code.
- **No premium leakage in UI.** No upsell prompts, badges or gates referencing premium in the MVP.
- **No proprietary AI infra.** AI integration uses BYOK (planned for premium); no LLM is hosted by us.

### Tests

- Rust: 59 passing (28 domain + services + 9 exporter + 6 storage extras + project_manager + integration + capabilities).
- Vitest: 19 passing (countWords, project store, document store, useShortcuts, useAutoSave).

[Unreleased]: https://github.com/OWNER/draffity/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/OWNER/draffity/releases/tag/v0.1.0-alpha
