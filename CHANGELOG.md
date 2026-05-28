# Changelog

All notable changes to Draffity are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Deferred to backlog futuro

- **Research browser embebido + bookmarks + captura web** (S6-01..03).
  Decisión consciente de no abordarlo en Sprint 6: la opción
  técnica entre `WebviewWindow` embebido vs iframe sandbox no
  estaba investigada y el riesgo era alto. Queda en backlog para
  un sprint dedicado cuando lo retomemos.

### Deferred to v0.8

- **Custom fonts** (S6-10, P2). Dropdown con fuentes del sistema
  más upload de TTF/OTF guardado en una tabla `media`. Depende de
  cómo resolvamos imágenes inline (S4-02), que también necesita
  storage de blobs — los hacemos juntos.

### Deferred (arrastres de Sprint 5)

- **`specta` para autogenerar tipos Rust↔TS** (S5-07). El payoff
  llega cuando tengamos ~30+ types. Cuando aterrice también cierra
  F1-12 del v1.
- **Pool de conexiones SQLite** (S5-08+09). Contención hipotética
  con un único usuario; sin benchmark que muestre cuello, es churn.
  Cuando aparezca storage premium o sync remoto.
- **Hot-swap de tier** (S5-10, P1). Sólo gana valor con un tier
  premium real al que swapear.

### Deferred (arrastres anteriores)

- **Footnotes** (S4-03), **imágenes inline** (S4-02), **diff visual**
  (S4-05), **MD/DOCX import** (S4-06/07), **round-trip tests** (S4-09).
- **PDF export** (S1-02 originalmente Sprint 1).
- **Stats históricas con gráfico de 30 días** (S2-08).
- **Split editor** (S3-06).

## [0.7.0-beta] — 2026-05-28

Sprint 6 cerrado parcialmente (6 de 10 historias). Se diferencia
por backups automáticos confiables, plantillas reutilizables y
customización del entorno de escritura.

El research browser (S6-01..03) se difiere conscientemente al
backlog futuro — la decisión técnica entre WebviewWindow embebido
y iframe sandbox necesita una sesión propia. Custom fonts (P2) y
los arrastres arquitectónicos de Sprint 5 (specta, pool) siguen
en "Deferred".

### Added — Sprint 6

- **Backup automático diario** (S6-04, S6-05): nuevo
  `BackupService` trait + `LocalBackupService` (free) + `NoOpBackup`
  (premium-ready para `CloudBackupService` futuro). Copia
  `<app_data>/draffity.db` a `<app_data>/backups/` con nombre
  `YYYY-MM-DD-HHMMSS-{daily,monthly,manual}.db`. Política de
  rotación: 7 dailies + último de cada uno de los últimos 6
  meses + todos los manuales. Al iniciar la app corre
  `run_daily_maintenance` (idempotente: si ya existe un daily
  para hoy es no-op) y luego poda. Errores se loguean pero nunca
  bloquean el arranque. `restore_backup` toma un manual de
  seguridad de la DB actual antes de pisarla, así el usuario puede
  deshacer. IPC: `list_backups`, `create_manual_backup`,
  `restore_backup`, `prune_backups`. UI: nueva sección "Backups"
  en Settings con lista (id, kind, fecha, tamaño) y botones para
  crear manual y restaurar (con confirm).
- **Plantillas de usuario** (S6-06): `template_from_project` arma
  un `Template` a partir del proyecto activo — estructura del
  binder (DFS por position), sinopsis conservados, contenido
  descartado (las plantillas siembran docs vacíos). El id va
  prefijado con `user-` + ULID para no chocar con built-ins; los
  documentos `trashed` no se incluyen. IPC:
  `save_project_as_template` + `delete_user_template`. UI: botón
  "Guardar como plantilla" en el header del proyecto con dialog
  (nombre + descripción opcional).
- **Loader de plantillas extendido** (S6-07):
  `UserTemplatesLoader` escanea `<app_data>/templates/user/*.json`
  en cada call (sin cache; se accede sólo al abrir el wizard).
  `LayeredTemplatesService` mergea built-in + user detrás del
  mismo trait — el wizard ve una lista única sorteada por nombre.
  Cambio cero en `ProjectManager` ni en `NewProjectWizard`.
- **CSS personalizado del editor** (S6-09): textarea en Settings
  persiste un snippet CSS en la tabla `settings`;
  `TipTapEditor` lo inyecta como hoja de estilos runtime con
  `sanitizeUserCss` que strippea `</style>`, `@import` y `url()`
  para evitar escape del bloque y carga de recursos remotos. Cap
  de 4 KB. El placeholder muestra el patrón `.tiptap-content`
  como prefijo de selector.
- **Atajos de teclado personalizables** (S6-08): refactor de
  `useShortcuts` — ahora se llama con un dict `{action: handler}`
  en lugar de `{combo: handler}`. La traducción action→combo vive
  en `useKeybindingsStore` (Pinia), que carga el dict persistido
  en `settings.editor.keybindings` y cae a `DEFAULT_BINDINGS` si
  no hay nada guardado. Action ids son estables; combos legacy
  se descartan si la action ya no existe (tolerante a upgrade).
  Settings suma una sección con `KeybindingsEditor`: por cada
  action muestra la combinación actual + botón "cambiar" que
  captura la próxima tecla (Esc cancela) y botón "por defecto".

### Architecture / migrations

- Sin migraciones nuevas — backups son archivos en disco; user
  templates JSON; CSS y keybindings en `settings`.
- `ServiceBundle` y `AppState` ahora cargan `backup` y
  `user_templates` (este último separado del `templates` trait
  para que el IPC pueda escribir sin downcasts).
- Patrón premium-ready aplicado a backup: `BackupService` trait
  más `NoOpBackup` stub. `CloudBackupService` futuro implementa el
  mismo trait sin tocar `lib.rs::setup`.

### Tests

- Rust: 126 verdes (112 previos + 8 backup + 6 user_templates).
- Vitest: 30 verdes (sin cambio).
- Playwright: 3 specs sin cambios.

## [0.6.0-beta] — 2026-05-28

Sprint 5 mayormente cerrado (6 de 10 historias): output listo para
publicar. La app deja de exportar con defaults rígidos y pasa a
tener un diálogo de export completo, EPUB con portada/imagen,
DOCX con TOC navegable y title page, e importación BibTeX +
citas inline con `[@key]` resueltas a `(Apellido, año)`.

Los 4 restantes son refactors de arquitectura/DX (specta, pool,
hot-swap) que se trasladan a v0.7 con justificación de costo en
"Deferred to v0.7" — ningún bloque que entregue valor al usuario
queda fuera del release.

### Added — Sprint 5

- **`ExportConfig` serializable + persistencia por proyecto** (S5-02):
  struct nuevo en `services/exporter/config.rs` con título override,
  autor, fuente, tamaño página, márgenes (mm), TOC sí/no, title page
  sí/no, separador de escena (Stars/Dashes/Blank/Custom) y cover
  image path. Persiste vía tabla `settings` con clave
  `export_config:<project_id>` (JSON). Comandos nuevos
  `get_export_config` / `set_export_config`. El trait
  `ExportService::export` ahora recibe `&ExportConfig`; defaults
  preservan la conducta previa.
- **Diálogo de export extendido** (S5-01): rediseño completo de
  `ExportDialog.vue` en 3 fieldsets colapsables (Contenido,
  Apariencia, Maquetación) + sección EPUB condicional cuando se
  elige ese formato. Carga la config persistida al abrir; al
  exportar, si está tildado "recordar para este proyecto", la
  guarda con `setExportConfig`. Todos los textos pasan por
  vue-i18n (ES + EN).
- **EPUB con cover image** (S5-03): nuevo file-picker en el dialog
  acepta jpg/png/gif/webp; el exporter detecta el MIME por
  extensión y embebe el blob vía `add_cover_image` de
  `epub-builder`. Además ahora honra `title_override` (sobrescribe
  `dc:title`), `author` (de config o `project.metadata.author`),
  `include_toc` (inline TOC) e `include_title_page`
  (condiciona el `title.xhtml`).
- **DOCX con TOC autogenerado + title page** (S5-04): el render
  emite un campo `TableOfContents` (heading levels 1-6, con
  hyperlinks y flag `dirty()` para que Word actualice al abrir).
  La portada — antes siempre presente — pasa a gobernarse por
  `include_title_page` e incorpora autor bajo el título.
  TOC y title page cierran con page break para que el manuscrito
  arranque limpio.
- **BibTeX import + tabla `citations`** (S5-05): migración 007
  aditiva agrega la tabla nueva con columnas id/project_id/key/
  entry_type/fields_json + `UNIQUE(project_id, key)` para upsert
  seguro y cascade delete. Nuevo dominio `Citation` con helpers
  para autor/año, storage submodule que hace upsert batch atómico,
  `LocalBibliographyService` que parsea con la crate `biblatex`
  limpiando braces/quotes y normaliza fields a un map plano. IPC
  `import_bibliography` / `list_citations` / `list_citation_keys` /
  `delete_citation`. UI: `BibliographyDialog` accesible desde el
  header del proyecto, con import .bib (vía `tauri-plugin-fs`),
  DataTable con borrado y conteo de omitidas.
- **Citas inline como nodo TipTap** (S5-06): nuevo node `citation`
  inline-atom con attrs `citationKey` + `label`; el label se
  pre-resuelve al insertar a `(Apellido, año)` usando el store
  `useCitationsStore`. El HTML serializado lleva el label dentro
  del `span data-citation-key`, así los 3 exporters (md/docx/epub)
  lo recogen como texto plano sin lógica nueva. Toolbar suma botón
  "Insertar cita" que abre `CitationPickerDialog` con búsqueda
  incremental por clave/autor/título.

### Architecture / migrations

- Migración 007 aditiva: tabla `citations` + índice por
  `project_id`. Backwards-compatible — proyectos previos siguen
  funcionando sin entries.
- Patrón premium-ready aplicado a la bibliografía:
  `BibliographyService` trait + `LocalBibliographyService` (free).
  Premium puede sumar `RemoteBibliographyService` (Zotero, etc.)
  sin tocar core.
- `ServiceFactory` extendido con `bibliography` — sigue el patrón
  de `LocalExporter` y queda listo para hot-swap por tier cuando
  llegue S5-10.
- Citas se persisten como HTML inline (no como entidad referenciada
  por id) porque el label es estable y los exporters solo ven HTML.
  Si la bibliografía se actualiza, el editor ofrece refresh
  manual (TODO en v0.7).

### Fixed

- Dos lints nuevos de clippy 1.95 (`derivable_impls`,
  `cloned_ref_to_slice_refs`) que aparecieron al añadir
  `PageSize`/`SceneSeparator`; usamos `#[derive(Default)]` con
  `#[default]` y `std::slice::from_ref`.

### Tests

- Rust: 112 verdes (87 previos + 4 ExportConfig + 4 EPUB + 4 DOCX +
  5 bibliografía + 3 citation storage + 5 citation domain).
- Vitest: 30 verdes.
- Playwright: 3 specs sin cambios.

## [0.5.0-beta] — 2026-05-28

Sprint 4 parcial: el editor empieza a parecerse a un procesador de
texto. Tablas TipTap con resize de columnas, spellcheck que sigue
el idioma de la UI, y el documento ahora persiste su estado
canónico de ProseMirror en JSON (no solo el HTML render-cache).
El alcance original del sprint era más ambicioso — ver "Deferred
to v0.6" para los bloques que se trasladan.

### Added — Sprint 4

- **Tablas en el editor** (S4-01): se añade el conjunto de
  extensiones de tabla de TipTap (v2.27.x, peer-compatible con el
  resto de TipTap v2.10), incluyendo TableRow, TableHeader y
  TableCell. Header row habilitada por defecto, columnas
  redimensionables por drag (`resizable: true`). Toolbar con 4
  botones nuevos: insertar tabla, agregar fila, agregar columna,
  eliminar tabla. Solo los 3 de edición se muestran cuando el
  cursor está dentro de una tabla (`isInTable` computado).
  Estilos scoped en TipTapEditor con handle de resize visible al
  hover.
- **Spellcheck dinámico por locale** (S4-08): el atributo `lang`
  del editable ProseMirror se sincroniza con `useI18n().locale`
  via watch directo sobre `editor.view.dom`. Cambiar idioma de UI
  conmuta el diccionario nativo del WebView sin recargar.
- **Persistencia canónica como JSON** (S4-04): migración 006
  aditiva agrega `content_json TEXT` a `documents`. El editor
  ahora hace dual-write — emite `update:modelValue` (HTML render-
  cache para export y FTS) y `update:modelValueJson` (estado
  ProseMirror canónico). Al rehidratar, `initialContent()`
  prefiere JSON cuando existe; HTML solo como fallback. Previene
  pérdida de atributos no-HTML como anchos de columna de tabla
  entre sesiones. `update_document` (IPC + trait) acepta el campo
  nuevo; `DocNode.contentJson` añadido a shared-types.

### Architecture / migrations

- Migración 006 aditiva: `ALTER TABLE documents ADD COLUMN
content_json TEXT`. `row_to_document` tolera la columna ausente
  para queries que no la pidan (backwards-compatible).
- El COLS constante de `storage/documents.rs` ahora lista
  `content_json` explícitamente; no se filtra fuera de los
  callsites internos. El SELECT de snapshots se actualiza en
  paralelo para no romper el restore.
- Patrón de doble emit en TipTapEditor habilita que tipo de
  ediciones futuras (footnotes, images, custom nodes) se persistan
  sin tocar el storage layer — el JSON ya es la fuente de verdad.

### Fixed

- 2 lints nuevos de clippy 1.95: `DocumentStatus` ahora usa
  `#[derive(Default)] + #[default]` en vez de impl manual;
  `reorder_documents` en test usa `std::slice::from_ref(&ch)` en
  vez de `&[ch.clone()]`. Sin cambios de comportamiento.

### Tests

- Rust: 87 verdes (los mismos del sprint anterior, callsites de
  `update_document` actualizados al 4to argumento `content_json`).
- Vitest: 30 verdes.
- Playwright: 3 specs sin cambios.

## [0.4.0-beta] — 2026-05-29

Sprint 3 cerrado: vistas tipo Scrivener. La app deja de ser solo
editor + binder y suma corkboard (cards con sinopsis), outliner
(tabla edit-in-place) y scrivenings (concat read-only de folder).

### Added — Sprint 3

- **Synopsis como campo de primera clase** (S3-01): migración 005
  agrega `synopsis TEXT` nullable a documents. El template_seed
  ahora lo popula ahí (antes lo metía en content como `<p>...</p>`).
  Inspector tiene Textarea auto-resize con debounce 400ms.
  IPC `set_document_synopsis` normaliza trim + empty→null.
- **Toggle de vista Editor/Corkboard/Outliner** (S3-02): ui store
  `projectViewModes` persiste el modo por proyecto en localStorage.
  `ProjectViewToggle.vue` SelectButton con 3 iconos en el header.
- **Composable useDocumentSummary** (S3-05): derivaciones
  compartidas (wordCount, progress, hasSynopsis, isFolder, etc.)
  para que card y row no dupliquen lógica.
- **CorkboardView** (S3-03): grid responsive de cards con title +
  synopsis (line-clamp 4) + word count + progress + tags + status
  dot. Click selecciona el doc. Sin drag-reorder en esta entrega —
  los usuarios reordenan vía drag&drop del binder (S1-01 sigue
  vigente). Drag-reorder en corkboard llega como follow-up cuando
  evaluemos sortablejs vs alternativa.
- **OutlinerView** (S3-04): DataTable PrimeVue con 5 columnas
  (title, synopsis, words+goal, status select, tags). Title y
  synopsis editan in-place (InputText / Textarea). Status edita
  inline con Select. Row click selecciona.
- **Scrivenings mode** (S3-07): cuando el documento seleccionado es
  folder, el panel central muestra `ScriveningsView` con DFS
  pre-order de descendientes concatenados read-only (folders como
  sub-headings, leafs como secciones con HR separador). Sale del
  storage HTML de TipTap vía v-html (fuente confiable).

### Added — Tests

- **S3-08 E2E**: spec `project-views.spec.ts` verifica que el toggle
  Editor → Corkboard → Outliner renderiza los componentes
  correctos y que la lista de documentos persiste cross-vista.
  Selectores aria-label tolerantes a locale ES/EN.

### Architecture / migrations

- Migración 005 aditiva: `ALTER TABLE documents ADD COLUMN synopsis
TEXT`. Documents pre-existentes se quedan con content
  HTML-escapado (no se migra de vuelta) — solo afecta a templates
  nuevos. row_to_document tolera la columna ausente con fallback.
- `useDocumentSummary` es el patrón canónico para nuevas vistas
  derivadas: composable que toma `Ref<DocNode|null>` y expone
  computeds.

### Tests

- Rust: 87 verdes (85 previos + 2 nuevos: synopsis round-trip y
  seed test actualizado).
- Vitest: 30 verdes (sin cambio).
- Playwright: 3 specs (dashboard, onboarding, project-views).

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

[Unreleased]: https://github.com/OWNER/draffity/compare/v0.7.0-beta...HEAD
[0.7.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.7.0-beta
[0.6.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.6.0-beta
[0.5.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.5.0-beta
[0.4.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.4.0-beta
[0.3.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.3.0-beta
[0.2.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.2.0-beta
[0.1.0-alpha]: https://github.com/OWNER/draffity/releases/tag/v0.1.0-alpha
