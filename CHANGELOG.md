# Changelog

All notable changes to Draffity are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Deferred to v0.4

- **PDF export** (S1-02 del Sprint 1). Sigue pendiente: la integración
  con `WebviewWindow::print()` necesita un preview en ventana propia
  con auto-print que en Tauri 2 requiere asset protocol + cleanup
  cuidadoso. Bloque dedicado en Sprint 3.
- **Stats históricas con gráfico de 30 días** (S2-08 del Sprint 2).
  Bloqueado por decisión de librería de charts (Chart.js vs ECharts
  vs custom SVG). El backend de `daily_writing` y el endpoint llegan
  primero; el gráfico, después.

## [0.3.0-beta] — 2026-05-28

Sprint 2 cerrado: productividad del escritor. La app pasa de "editor
con binder al lado" a un entorno de trabajo con pipeline de status,
tags, objetivos de palabras y temporizador Pomodoro.

### Added — Sprint 2

- **Document status** (S2-01, S2-02, S2-03): pipeline
  `draft → revised → final → trashed` con migración 003 aditiva.
  Selector en Inspector (PrimeVue Select), badge de color como punto
  pequeño junto a cada nodo del binder. `DocumentStatus` enum en
  dominio Rust con tests round-trip.
- **Tags por documento** (S2-04): tabla `document_tags(document_id,
tag)` con PK compuesta + cascade delete. Subquery
  `json_group_array(tag)` embebido en el SELECT de documentos evita
  N+1. Chips PrimeVue en Inspector, dropdown de filtro en el binder
  con preservación de ancestros. IPC `set_document_tags`,
  `list_project_tags`.
- **Word count goals** (S2-05): columna `goal_words INTEGER` nullable
  en `projects` y `documents` (migración 004). `GoalProgress.vue`
  reutilizable con barra de progreso (colores por tramo), edición
  inline con `InputNumber`, modo compact para headers. Inspector
  muestra goal del documento, header del ProjectView muestra goal
  del proyecto (totalWordCount / project.goalWords).
- **Session word goal** (S2-06): `uiStore.sessionStartTotal` capturado
  por ProjectView al cargar, `sessionGoal` persistido en localStorage.
  Widget compact en AppShell cuando hay documentos cargados.
- **Pomodoro / writing timer** (S2-07): composable `useWritingTimer`
  con state machine idle/work/break/paused + tick 1s. focusMin y
  breakMin persistidos. WebAudio beep al cambiar de fase (sin bundle
  de audio file). `PomodoroWidget.vue` en AppShell: chip con color
  por fase, MM:SS mono, popover de ajustes + counter de sesiones.
- **Typewriter mode** (S2-09): composable `useTypewriterScroll` que
  centra verticalmente la línea actual del cursor en el scroll
  container. Toggle en Settings con `ToggleSwitch`. Persistido.
- **Atajos navegación binder** (S2-10): `Ctrl+,` doc anterior,
  `Ctrl+.` doc siguiente (orden natural de `list_documents`).

### Added — Tests

- **S2-11**: 11 tests Vitest para `useWritingTimer` cubriendo el
  state machine completo con `vi.useFakeTimers()`.

### Changed — Architecture

- Constante `COLS` introducida en `storage/projects.rs` (mirror del
  patrón ya usado en `documents.rs`). Añadir columnas futuras es un
  cambio de una línea.
- `row_to_document` y `row_to_project` toleran columnas ausentes con
  fallback graceful (status, goal_words) — backwards-compatible con
  queries que no las pidan.

### Tests

- Rust: 85 verdes (73 previos + 6 status + 4 tags + 2 goals).
- Vitest: 30 verdes (19 previos + 11 timer).

## [0.2.0-beta] — 2026-05-27

Cierre del Sprint 1: deuda del alpha + base sólida para los sprints
de productividad. Premium-aditivo intocado.

### Added — Sprint 0 (architecture guardrails)

- `CLAUDE.md` (no versionado) con workflow obligatorio antes de codear,
  patrones canónicos, antipatrones y reglas Rust/Vue específicas.
- `docs/ADR/` con README + template MADR-ligero + 3 ADRs retroactivos
  (Tauri sobre Electron, SQLite canónica única vs por proyecto,
  premium aditivo vía traits).
- Patrones canónicos documentados en `docs/ARCHITECTURE.md` con
  referencias al código que los materializa.
- `vue/no-bare-strings-in-template` como error en ESLint: cualquier
  string nueva en `<template>` debe pasar por `vue-i18n`. Allowlist
  con tokens UI universales (H1-H6, B/I/U/S, ms/px/em, teclas).
- Scripts `coverage:rs`, `coverage:rs:summary`, `coverage:ts` en
  package.json + doc en CONTRIBUTING.

### Changed — Sprint 0 refactors

- `services::storage` pasa de un único archivo de 1115 líneas a un
  módulo con 8 archivos hermanos (`projects`, `documents`, `snapshots`,
  `settings`, `stats`, `row_mappers`, `template_seed`). El trait
  `StorageService` queda intacto; el impl es un delegador fino.
- `ServiceFactory::build(tier, app_data_dir) -> ServiceBundle`: el
  wiring de servicios sale de `lib.rs::setup`. `AppState::from_bundle`
  compone el state. `lib.rs::run` queda en ~13 líneas. Habilita
  hot-swap de tier sin tocar bootstrap.
- `NewProjectWizard.vue` se divide en 3 step components controlled-
  by-parent: `WizardStepTemplate`, `WizardStepMetadata`,
  `WizardStepConfirm`. El orquestador queda en 236 líneas (bajo el
  límite suave de 250).
- Tests Rust: `expect("razón")` en fixtures de bootstrap.

### Added — Sprint 1 features

- **Drag & drop en el binder** (S1-01): reordenar y mover documentos
  entre carpetas vía arrastre. Persistencia atómica con reindex de
  posiciones; rollback si algún id falla.
- **Búsqueda full-text por proyecto** (S1-04 + S1-05): FTS5 sobre
  `documents` con migración 002, triggers de sincronización
  AI/AD/AU, tokenizer `unicode61 remove_diacritics 2` (canción
  matchea cancion). Dialog modal Ctrl+Shift+F con resultados
  resaltados via `<mark>`.
- **Find & Replace en documento** (S1-03): Ctrl+F (find) y Ctrl+H
  (replace) abren una barra anclada al editor. Navegación con
  Enter/Shift+Enter, contador "N/M", Replace y Replace all,
  case-insensitive. Recompute reactivo cuando el editor cambia.
- **Focus mode** (S1-07): F11 o botón en header oculta binder e
  inspector. El editor crece al 100%. focusMode store ahora
  conectado a la UI.
- **Onboarding paso final lanza el wizard** (S1-08): el último slide
  ("Crear mi primer proyecto") setea un flag one-shot en `uiStore`
  que el Dashboard consume al montar para abrir el NewProjectWizard.

### Added — CI

- **epubcheck en CI** (S1-06): job nuevo `validate-epub` que instala
  Temurin 21, descarga epubcheck 5.1.0, genera un fixture EPUB via
  `cargo run --example fixture_epub` y lo valida. Falla CI si el
  output no cumple la spec.

### Added — Docs

- `docs/USER-GUIDE.en.md`: traducción completa al inglés.
- USER-GUIDE.md (ES): sección de búsqueda + atajos nuevos.
- README: estado actualizado a v0.2.0-beta + links a ambas guías.

### Architecture / tests

- **Patrones canónicos** ahora también incluyen Strategy aplicado a
  exporter, Event bus para reaccionar sin acoplar (búsqueda no
  necesita evento, pero futuras features sí).
- Tests Rust: **73 verdes** (65 previos + 3 nuevos para
  `reorder_documents` + 5 nuevos para `search_documents`).
- Vitest: 19 verdes (sin cambio — `useFindReplace` queda sin tests
  unitarios por dependencia del editor TipTap montado; Sprint 2).

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

[Unreleased]: https://github.com/OWNER/draffity/compare/v0.3.0-beta...HEAD
[0.3.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.3.0-beta
[0.2.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.2.0-beta
[0.1.0-alpha]: https://github.com/OWNER/draffity/releases/tag/v0.1.0-alpha
