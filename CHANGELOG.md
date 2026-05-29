# Changelog

All notable changes to Draffity are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Desarrollo del **backlog v4** (interno): cerrar brechas competitivas + capa
premium (IA BYOK + voz). Aún sin release versionada.

### Added — Épica E: foundations premium (E-01..E-10)

- **Secrets en keyring del SO** (E-01): trait `SecretStorage` +
  `KeyringSecretStorage` (Windows Credential Manager / macOS Keychain /
  Linux Secret Service) + `InMemorySecretStorage` para tests. Las API keys
  BYOK y la licencia nunca tocan la tabla `settings` en texto plano.
- **Tier hot-swap** (E-06, cierra el arrastre S5-10): `PremiumTier` +
  `MutableTier` (con `set_tier` en el trait) → activar premium cambia las
  capabilities en vivo, sin reiniciar, a través del único `Arc<dyn TierService>`
  compartido.
- **Licencia premium offline** (E-07): `LicenseValidator` + `Ed25519Validator`
  (firma asimétrica; sólo la clave pública viaja en la app) +
  `DisabledLicenseValidator` para builds OSS sin clave. Comandos
  `activate_premium` / `deactivate_premium` / `get_premium_status` + restore
  al arranque desde el keyring. Pubkey por env `DRAFFITY_LICENSE_PUBKEY`.
- **Contratos premium-ready** (E-03/E-04): `AIService` con `stream_complete`
  por callback sink (object-safe, sin runtime async); `ASRService` con
  `Transcript` + streaming; nuevo `TTSService` + `NoOpTTS`.
- **Capabilities granulares** (E-05): `ai_inline`, `ai_validators`,
  `voice_dictation`, `voice_tts`, `voice_notes` + umbrellas.
- **Sidecar infra** (E-02): helper `run_sidecar` sobre `tauri-plugin-shell`
  (binarios los provee la Épica H).
- **Event bus tipado** (E-09): enum `AppEvent` mapeando a las consts estables.
- **Telemetría local-only** (E-10): appender `premium-events.log` filtrado por
  target `ai_events`/`voice_events`. Cero red.
- **UI premium** (E-08): activación de licencia + secciones IA/Voz en Settings,
  gateadas por capability (sin leakage en free).

### Added — Épica F: editor IA inline con BYOK OpenRouter (F-01..F-13)

- **Motor OpenRouter BYOK** (F-01): `ByokAIService` vía `reqwest::blocking` +
  parseo SSE (función pura testeada), retry con backoff, errores
  `AppError::AiProvider`. Gateado por tier + key en cada llamada (hot-swap sin
  rebuild). Key en keyring.
- **Memoria del proyecto engram-aligned** (F-03): `ProjectMemoryService` léxico
  que arma contexto con codex mencionado (semántica) + pasajes FTS5 (episódica)
  - sinopsis, recortado a budget. Embeddings = upgrade opt-in post-v4 detrás del
    mismo trait. Estimador de tokens heurístico (F-02).
- **Acciones inline** (F-08..F-11): Continuar / Expandir / Reescribir (6
  sub-modos + custom) / Describir. Prompts curados en `ai_prompts.rs`.
- **Streaming + preview + accept/reject** (F-06/F-07): bubble menu flotante
  sobre la selección (`@floating-ui/dom`), preview con streaming en vivo por
  evento `ai.suggestion.received`, diff rojo/verde para rewrite/expand, Enter
  acepta / Esc descarta+cancela.
- **Slash command** (F-05): `/` al inicio de línea continúa desde el cursor.
- **Historial de IA** (F-12): migración 012 + `ai_history`; sólo se persiste lo
  aceptado. Comandos `ai_record_accepted` / `list_ai_history`.
- **Cost meter** (F-13): store de tokens del mes (real usage de OpenRouter,
  coincide con su dashboard) en Settings + link a la página de actividad.
- **Cancelación** (F-06): `AiCancelRegistry`; el sink deja de emitir al
  cancelar.

### Added — Épica G: validadores IA (G-01..G-10)

- **5 validadores** (G-04/05/06/07/08): coherencia de personajes, voz/tono y
  trama/temporalidad son model-backed (prompt con contrato JSON + parseo
  tolerante); repetición (n-gramas) y estilo (oraciones largas, adverbios, voz
  pasiva) son heurísticas locales sin red. Orquestador `OpenRouterValidators`
  detrás del trait `AIValidatorService`.
- **Codex coverage pre-check** (G-03): estima qué tanto del elenco aparente del
  texto está en el codex y avisa antes de correr si es escaso.
- **Persistencia** (G-02): migración 013 + tabla `ai_validations` (cascade con
  el documento). Reabrir el documento muestra el último reporte sin re-llamar
  al modelo.
- **Command layer**: `run_validators` (batch G-10, off-thread, un fallo por
  validador no aborta el resto), `check_codex_coverage`, `list_validations`.
- **UI** (G-09/G-10): `ValidationDialog` con toggles por validador (persistidos),
  aviso de cobertura, reporte por severidad con excerpt + sugerencia, y "ir al
  texto" que salta y selecciona el pasaje en el editor. Botón "Análisis IA" en
  el header, gateado por capability.
- **Diferido**: `CodexKind::Event` + campo `when` (G-00) como enriquecimiento
  opcional del validador de trama, que ya funciona con el codex existente.

### Added — Épica H: voz local (H-01..H-11)

- **Runtime de voz opt-in** (H-01/H-02): binarios + modelos viven en
  `<app_data>/voice/`, se descargan (streaming + checksum + escritura atómica)
  o se importan; nada en el instalador, ejecución por `std::process`.
- **Dictado** (H-03/H-04): `WhisperLocalASR` (spawn whisper.cpp, parseo JSON con
  timing) detrás del trait `ASRService`. Frontend: `useAudioRecorder` graba y
  resamplea a WAV 16 kHz mono, `useDictation` transcribe e inserta en el cursor,
  overlay con medidor de nivel, atajo Ctrl+Shift+M.
- **Lectura en voz alta** (H-05/H-06/H-07): `PiperTTSService` (spawn Piper → WAV
  → PCM) tras el trait `TTSService`; `useReadAloud` sintetiza oración por oración
  con highlight del pasaje actual, controles play/pausa/saltar/velocidad.
- **Notas de voz** (H-08/H-09/H-11): migración 014 (media gana
  `duration_ms`/`transcribed_text`/`is_voice_note`); grabar (reusa el grabador),
  reproducir y transcribir en background. `VoiceNotesDialog`.
- **Modelos/voces** se gestionan desde Settings → Voz (descarga con progreso +
  import de binario). `available()` honesto: la UI solo ofrece voz si el binario
  y un modelo/voz están realmente instalados.
- Motores **intercambiables**: cambiar de ASR/TTS = nueva impl del trait + una
  línea en `factory`; UI, comandos y composables intactos. Parsers puros
  (JSON whisper, WAV, encoder WAV, autopuntuación) unit-testeados.
- **Diferido**: vínculo codex↔nota de voz (H-10, P1).

### Added — Épica I: paridad binder/metadata (I-01..I-10)

- **Colecciones** (I-01..I-04): migración 015 (`collections` +
  `collection_documents`). Manuales (lista ordenada) y smart (query serializado
  resuelto en vivo por `CollectionQuery::matches`, lógica de dominio pura). UI:
  panel bajo el binder + editor con constructor de filtros (tags/estado/título).
- **Marcadores de color** (I-05/I-06): migración 016 (`labels` +
  `document_labels` M:N). Taxonomía coloreada por proyecto; `DocNode.label_ids`
  vía subquery `json_group_array`. Chips en binder/outliner/corkboard/inspector,
  asignación con MultiSelect, gestor con paleta y filtro de binder por label.
- **Campos personalizados** (I-08/I-09): migración 017 (`custom_fields` +
  `document_custom_values`). Kinds text/number/date/select con validación de
  opciones; `DocNode.metadata` vía `json_group_object`. Editor por kind en el
  inspector, gestor de definiciones y columnas ordenables en el outliner.
- **Carpeta de investigación** (I-10): migración 018 (`documents.is_research`
  aditiva). El subárbol research se excluye del conteo de palabras y del export
  por defecto (`strip_research` + `ExportConfig.include_research` opt-in). Toggle
  en inspector, indicador en binder, checkbox en el diálogo de export.
- **Diferido**: status configurable por proyecto (I-07). Rompería el enum tipado
  `DocumentStatus` (CHECK SQL, semántica de `trashed`, filtro de colecciones,
  dots, tipo specta) y se solapa con Labels, que ya da taxonomía de color por
  proyecto. Se revisará si aparece demanda real del _pipeline_ configurable.

### Added — Épica J: goals avanzados + linguistic focus (J-01..J-09)

- **Deadline + Pacemaker** (J-02/J-03): migración 019 (`projects.deadline`
  aditiva, epoch ms). `usePacemaker` (puro) calcula palabras/día = restantes /
  días, con estado según el avance de la sesión (ontrack/close/behind/overdue/
  done). `PacemakerWidget` en el header: chip de color + DatePicker para fijar /
  limpiar el plazo. Barra de progreso por documento en el outliner (J-01).
- **Meta diaria + racha de meta cumplida** (J-04/J-05): migración 020
  (`daily_writing` gana `goal_words` + `goal_met`). Meta diaria como setting
  backend; `record_daily` snapshotea la meta y recomputa `goal_met`.
  `WritingStats.goalMetStreak` = días consecutivos con meta cumplida. UI en
  Settings (input + racha). `no_trash_days` diferido (semántica ambigua).
- **Foco lingüístico** (J-06/J-07): extensión TipTap `LinguisticFocus` con
  detección pura y decoraciones (no toca el documento) — resalta adverbios
  (`-mente`/`-ly`), pasiva (heurística conservadora) y diálogo. Toggle en la
  toolbar, persistido; palabras extra configurables en Settings (J-07).
- **Mapa de repeticiones** (J-08): extensión `RepetitionHeatmap` local (sin IA)
  que marca palabras de contenido y bigramas repetidos con 3 niveles de calor.
- **Tiempo de lectura** (J-09): estimación palabras/wpm en el inspector, WPM
  configurable en Settings (default 200).

### Added — Épica K: compile, generador de nombres, composición v2 (K-01..K-10)

- **Generador de nombres** (K-06/K-07): dataset embebido de 12 orígenes
  (masc/fem/unisex), generador puro (`pickNames` con RNG inyectable),
  `NameGeneratorDialog` accesible desde el campo nombre del codex; import de
  listas custom `.txt`/`.json` persistidas en uiStore.
- **Modo composición v2** (K-08/K-09): superficie fullscreen distraction-free
  con ancho de papel, color de fondo y fade por párrafo (extensión
  `ParagraphFade`) configurables; barra de control hover-reveal, atajo
  Ctrl+Shift+F11 y Esc. Diferido: sonido typewriter, imagen de fondo, fade por
  oración/línea, persistencia por proyecto.
- **Split avanzado** (K-10): lock de panel (fija el doc) + bookmarks por panel
  (últimos docs abiertos) persistidos por proyecto.
- **Compile presets** (K-01..K-05): enfoque pragmático format-agnóstico —
  find&replace de export (`ExportConfig.findReplace`, aplicado al contenido en
  el comando export), front/back matter (migración 021 + reorden de docs raíz a
  los extremos), y 3 presets built-in (manuscrito/ebook/borrador) con selector
  en el ExportDialog. Diferido: layout por-DocumentType y wiring de
  `scene_separator` (K-04, requiere reescribir los renderers) y tabla de presets
  de usuario (K-01).

### Deferred to backlog futuro

- **Research browser embebido + bookmarks + captura web** (S6-01..03 →
  E-03). Decisión consciente: la opción técnica entre `WebviewWindow`
  embebido vs iframe sandbox no estaba investigada y el riesgo era
  alto. Queda en backlog para un sprint dedicado.

### Deferred (arrastres de Sprint 5)

- **Pool de conexiones SQLite** (S5-08+09 → E-01). Contención
  hipotética con un único usuario; sin benchmark que muestre cuello,
  es churn. Cuando aparezca storage premium o sync remoto.
- **Hot-swap de tier** (S5-10, P1 → E-02). Sólo gana valor con un
  tier premium real al que swapear.

## [0.12.0-beta] — 2026-05-28

Sprint D cerrado al 100% (7 historias). Foco en **DX, arquitectura y
calidad**: tipos auto-generados con drift gate, coverage floors,
crash reporting opt-in, tema alto contraste, política de privacidad
y ToS, y a11y E2E. Sin features visibles nuevas — el output es un
codebase con gates más fuertes y un UX que respeta a usuarios con
necesidades de accesibilidad y privacidad explícitas.

### Added — Política de privacidad + ToS (D-06, S8-11)

- Cuatro markdowns en `docs/` como fuente canónica:
  `PRIVACY-POLICY.md`, `PRIVACY-POLICY.en.md`, `TOS.md`, `TOS.en.md`.
- Copia bajo `apps/ui/src/assets/legal/` que se importa vía Vite
  `?raw` y se muestra en `LegalDialog.vue` en monoespaciado. Sin
  dep de markdown renderer: el texto es prosa breve y se lee bien
  pre-wrapped.
- Settings agrega sección "Legal" con dos links que abren el dialog
  en el idioma activo del editor.

### Added — High-contrast theme (D-05, S8-09)

- `ThemeMode` crece a `'high-contrast'` (riding on top of dark, no
  como toggle independiente). `<html>` ahora lleva dos canales
  ortogonales: `app-dark` + `app-high-contrast`.
- PrimeVue v4 lee `--p-*` CSS vars en runtime, así que el override
  en `main.css` retunea toda la cascada (surface, primary, content)
  sin forkear el preset. Paleta amarillo saturado `#ffd700` sobre
  negro puro (WCAG-AAA), focus ring de 3px, bordes opacos forzados.
- Settings agrega cuarta opción "Alto contraste" / "High contrast"
  en el `SelectButton` de tema.

### Added — Crash reporting opt-in (D-04, S8-10)

- Trait `CrashReporterService` + `NoOpCrashReporter` +
  `LocalFileCrashReporter` (escribe reportes JSONL bajo
  `<app_data>/crash-reports/` — stand-in hasta que el owner
  provisione Sentry self-hosted). El factory mira la env var
  `DRAFFITY_SENTRY_DSN` del build: si está seteada wirea el
  local-file reporter para ejercer el pipeline end-to-end; si no,
  `NoOp`.
- Settings agrega `ToggleSwitch` "Send crash reports" sólo cuando
  el reporter es activo. Default OFF, persistido en settings con
  clave `crash_reporting.enabled`; `lib.rs` restaura el flag al
  startup. Comandos Tauri `get_crash_reporting_status` +
  `set_crash_reporting_enabled`.

### Added — a11y E2E con axe-core (D-07, S8-06)

- Nueva spec `apps/ui/e2e/a11y.spec.ts` con dos escaneos via
  `@axe-core/playwright` filtrados a tags WCAG 2.0/2.1 AA.
  Cualquier violación rompe el job e2e existente — el descubrimiento
  ya cubre `.spec.ts` del directorio, así que no hace falta wiring
  extra de CI.
- `best-practice` queda fuera a propósito: incluye reglas que no son
  conformance failures (heading-order sobre decisiones de diseño) y
  ensuciaría el gate sin sumar valor.

### Added — Coverage gates (D-02 + D-03, S8-04 + S8-05)

- **Rust (D-02)**: nuevo job `coverage-rs` en `ci.yml` usa
  `cargo-llvm-cov` con `--fail-under-lines 80`. Filtra wiring
  (`commands/`, `lib.rs`, `state.rs`, capabilities, events, logging)
  más tests y examples para medir sólo `domain/` y `services/`.
- **TS (D-03)**: `vitest --coverage` vía `@vitest/coverage-v8`,
  config en `vite.config.ts` con scope a `composables/` y
  `stores/`. Thresholds al floor actual (lines 30, functions 55,
  statements 30, branches 70) por honestidad: el target del backlog
  es 70% lines y se ratchetea a medida que se sumen tests para los
  stores/composables sin cobertura. El gate hoy bloquea regresiones.

### Added — Specta gen-types con drift gate (D-01, S5-07, parcial)

- `specta` v2 + `specta-serde` + `specta-typescript` wired. Bin
  `gen-types` exporta los enums simples del dominio (`ProjectStatus`,
  `DocumentType`, `DocumentStatus`, `CodexKind`) a
  `packages/shared-types/src/generated.ts`.
- CI corre el bin después de `cargo test` y falla con `git diff` si
  `generated.ts` no coincide con lo committeado — agregar un
  `serde::Serialize` en `domain/` sin regenerar rompe la build.
- Scope intencionalmente acotado: los structs con
  `serde_json::Value` (`Project`, `ProjectInput`, `CodexInput`,
  etc.) quedan en `index.ts` como manual hasta que
  `specta-typescript` crezca soporte cleano para
  `Record<string, unknown>`. El `index.ts` re-exporta los cuatro
  enums desde `generated.ts` así callers no notan la migración.

## [0.11.0-beta] — 2026-05-28

Sprint C cerrado al 100% (5 historias). Cierra el ciclo de export
con import bidireccional, suma diff visual entre snapshots y termina
el formato PDF pendiente desde Sprint 1. Toda nueva entrada se
apoya en el patrón premium-ready (trait + impl + NoOp) consolidado
en Sprints A/B.

### Added — Markdown import (C-01, S4-06)

- **Nuevo trait `ImportService`** + `LocalImporter` (dispatch por
  formato) + `LocalMarkdownImporter` (struct enfocado, útil para
  callers/tests que sólo necesitan Markdown). Sigue el patrón
  premium-ready: una futura `CloudImporterService` implementa la
  misma surface.
- **Parser** con `pulldown-cmark` v0.12: YAML frontmatter mínimo
  (`title:` se usa como project title cuando está presente), split
  por `#`-headings con jerarquía H1→Folder, H2→Chapter, H3→Scene.
  Skip del primer H1 sólo cuando sirve como título de proyecto
  (único H1 al tope con headings nested debajo).
- **Footnotes round-trip**: `[^id]: body` se levantan en una pasada
  previa y los `[^id]` inline se reescriben como
  `<sup data-footnote-id="…" data-footnote-content="…">†</sup>`
  para que el editor las reconozca al primer save.
- **Tauri command** `import_project` + `supported_import_formats`;
  storage suma `create_project_from_import` que inserta proyecto +
  tree atómico. Dashboard agrega botón "Importar…" que abre el
  file picker y navega al proyecto creado.

### Added — DOCX import (C-02, S4-07)

- **Reader** con `roxmltree` v0.20: abre el ZIP, lee
  `word/document.xml` y recorre `<w:p>` paragraphs. Heading levels
  detectados desde `<w:pStyle>` con soporte para variantes en
  inglés (`Heading1..6`) y español (`Ttulo1..6` tras strip del
  acento).
- **Inline marks**: cada `<w:r>` aporta texto + estado de
  `<w:rPr>` que se traduce a `<strong>` / `<em>` / `<u>` / `<s>` /
  `<code>` — el subset que el editor maneja. `<w:val="none">` y
  `<w:val="false">` se interpretan como desactivaciones explícitas.
- **Tree builder** comparte la misma regla de skip del primer H1
  con el importer de Markdown, así proyectos con un sólo Heading1
  como portada producen la misma forma de árbol independiente del
  formato fuente.
- Tablas y footnotes quedan fuera del MVP: no son lossless aún y
  exponer soporte parcial sería confuso. Documentado en el cabezal
  del módulo.

### Added — PDF export (C-05, S1-02)

- **Nuevo renderer** `services/exporter/pdf.rs` que genera HTML
  standalone con CSS print-friendly (`@page A4`, `page-break-before`
  por capítulo, TOC opcional, sección de notas al pie por capítulo,
  imágenes inline como data URIs). Script embebido dispara
  `window.print()` apenas el documento carga.
- **`ExportFormat::Pdf`** escribe extensión `.html` (más honesto
  que un `.pdf` con HTML adentro) y `ExportDialog` detecta el caso
  para abrir el archivo via `tauri-plugin-shell::open` — el
  navegador predeterminado lanza el diálogo de impresión del SO y
  el usuario elige "Guardar como PDF". `WebviewWindow::print_to_pdf`
  sólo está expuesto en macOS en Tauri 2, así que `window.print()`
  es la salida común a todas las plataformas.
- Sin nueva dep nativa de PDF: el output queda WYSIWYG con la
  maquetación del editor (Lora serif, márgenes 24mm).

### Added — Diff visual entre snapshots (C-04, S4-05)

- **Composable `useTextDiff`** con LCS line diff puro (sin
  `diff-match-patch` — 50KB minificado que no necesitamos a este
  nivel) + helper `htmlToLines` que normaliza HTML del editor a
  texto por párrafo. Cambios de marcas inline (bold/italic) no
  disparan diffs falsos — sólo cambios reales de texto.
- **Componente `SnapshotDiffView`** muestra dos columnas alineadas
  con rojo para removidos y verde para agregados, contador de +N
  −M, y placeholder cuando las versiones son idénticas.
- **`SnapshotsList`** recibe `currentHtml` y agrega botón
  "Comparar con actual" por cada versión. `Inspector` propaga el
  `doc.content` actual hacia abajo.

### Added — Round-trip tests (C-03, S4-09)

- **Integration suite** `tests/round_trip_integration.rs` con cinco
  escenarios:
  - Markdown preserva títulos, texto de body y jerarquía
    H1→H2→H3 del binder.
  - Markdown preserva footnotes (`data-footnote-content` vuelve
    idéntico al body).
  - DOCX preserva títulos + texto de body.
  - DOCX preserva marcas inline (bold/italic/under) que el editor
    soporta.
- **Cobertura tolerante** a presentación (whitespace, marker
  characters elegidos por el renderer) y estricta en
  identity-preserving content. Tablas e imágenes quedan
  documentadas como fuera de scope hasta que los importers crezcan
  ese soporte.

### Fixed

- **Markdown importer** trim_start de `\n\r` al body antes de
  `split_by_headings`: sin esto la blank line entre el cierre de
  frontmatter y el primer heading se promovía a una sección
  sintética "Intro" y rompía el skip_first_h1 al duplicar el
  conteo de H1s.

## [0.10.0-beta] — 2026-05-28

Sprint B cerrado al 100% (5 historias). Primer sprint del backlog v3
con foco en **features de editor** acumuladas como arrastres de
Sprints 2-6 originales: imágenes inline, fuentes personalizadas,
notas al pie, gráfico de hábito de escritura y editor partido. Toda
nueva funcionalidad se asienta sobre el patrón premium-ready (trait +
impl) consolidado en Sprint A.

### Added — Imágenes inline (B-01, S4-02)

- **Trait `MediaService` + `LocalMediaService` + `NoOpMedia`** con
  storage de blobs en `<app_data>/media/<project>/<sha256>.<ext>`.
  Dedupe por `(project_id, sha256)`: pegar la misma imagen dos veces
  reusa un único archivo. Migración 010 con tabla `media(id,
project_id, path_relative, mime, sha256, bytes, created_at)` +
  cascada delete desde proyectos.
- **Editor TipTap**: nodo `Image` con NodeView Vue que resuelve la
  Blob URL desde el `useMediaStore` (cache + in-flight dedupe +
  revoke en `reset()`). La HTML persistida nunca lleva `src` — solo
  `data-media-id` —, así el documento es portable y resistente a
  reinicio.
- **Export**: pre-pass `MediaBundle` lazy-resuelto en el comando
  `export_project`. Markdown emite `![alt](data:URI;base64,…)`; EPUB
  agrega los bytes como recursos + reescribe `src`; DOCX acepta el
  bundle pero queda con TODO explícito para emitir Pic (necesita
  parsing de dimensiones PNG/JPEG).
- **Tests**: 5 unit de `MediaBundle` + 7 de storage + 7 de servicio +
  integración real round-trip de imagen pegada → export → bytes
  decodificables en MD/EPUB.

### Added — Stats UI con gráfico 30 días (B-04, S2-08)

- **Migración 011** con tabla `daily_writing(date PK, words,
sessions, updated_at)` que acumula deltas positivos de palabras
  por día (las eliminaciones no restan — el chart mide progreso, no
  net change) más conteo de sesiones de save.
- **Domain helper** `count_words_in_html` (strip tags + tokens
  whitespace) y servicio `record_daily_writing` / `list_recent_daily_writing`
  con padding de días vacíos para que el chart siempre reciba
  exactamente N entradas.
- **Sparkline SVG puro** (`SparklineChart.vue`) sin librería externa
  en Settings junto al panel de rachas. Tooltips por barra +
  resumen "X palabras · Y días activos".
- **Pipeline**: `update_document` captura word-count previo de la
  doc, computa el delta tras el save (`saturating_sub` para no
  contabilizar bordes negativos) y delega a
  `record_daily_writing`. La persistencia es best-effort: un fallo
  de stats nunca bloquea el save del documento.

### Added — Footnotes con numeración automática (B-03, S4-03)

- **Nodo TipTap** inline atómico que persiste el cuerpo como
  atributo (`data-footnote-content`), así el HTML sigue siendo
  autocontenido. Marker visible en el editor es un dagger
  clickeable; al clickear dispatch `draffity:open-footnote` y
  ProjectView abre el `FootnoteDialog` para insertar/editar/eliminar.
- **Numeración a tiempo de export**: módulo
  `services/exporter/footnotes.rs` con `collect_footnotes` que
  recorre el HTML reemplazando cada `<sup data-footnote-content>`
  por un marcador específico del formato (1-indexado por capítulo
  en EPUB, por documento en Markdown) y devuelve las notas
  ordenadas para emitir la sección al pie.
- **Markdown**: `[^N]` inline + bloque `[^N]: contenido` al final del
  capítulo. **EPUB**: `<a epub:type="noteref">` + `<aside
epub:type="footnote">` con back-link `↩` por nota. **DOCX**:
  emite footnote references nativos vía `docx-rs::Footnote`, así
  Word renumera y maqueta al pie automáticamente.
- **Toolbar**: botón "Insertar nota al pie" (`pi pi-asterisk`) +
  aria-label.

### Added — Custom fonts (B-02, S6-10)

- **Picker en Settings** con dropdown agrupado: built-ins (Lora,
  Inter, JetBrains Mono), fuentes del sistema más comunes
  (Georgia, Garamond, Palatino, Helvetica, Verdana, Courier) y
  fuentes personalizadas subidas por el usuario al proyecto activo.
  Quick-toggle de 3 botones mantenido como atajo rápido.
- **Upload TTF/OTF**: reutiliza el `MediaService` de B-01 (mime
  `font/ttf` y `font/otf` ya soportados). El picker filtra por
  `mime.startsWith('font/')` los media del proyecto activo.
- **Aplicación runtime**: el editor expone `--editor-font-family`
  como variable CSS; para fuentes custom se inyecta `@font-face`
  con la Blob URL resuelta por `useMediaStore`. Al cambiar la
  fuente, el editor intercambia el blob sin recargar el documento.
- **Persistencia**: setting `editor.font_family` (CSS family string
  libre) + `editor.font_custom_id` opcional. El ajuste legacy
  `editor.font` (serif/sans/mono) migra al primer load.

### Added — Split editor (B-05, S3-06)

- **Toggle en cabecera de proyecto** (`pi pi-clone`) que envuelve
  el editor primario en un Splitter horizontal junto a un
  `SplitSecondaryPane`. El panel secundario incluye su propio
  `Select` para elegir cualquier documento del proyecto (excepto el
  primario) y se cierra desde dentro del panel.
- **Autosave coordinado**: el panel primario sigue usando el
  document store (selección compartida con binder e inspector), el
  secundario hace fetch directo (`ipc.getDocument`) + save sin
  tocar el estado del primario. Indicador "Saving" propio.
- **Persistencia** por proyecto: `splitSecondaryIds` en `uiStore`
  guarda el doc id del panel secundario para que cada proyecto
  recuerde el último layout al volver.

### Changed — Export signature

- `ExportService::export` ahora acepta `&MediaBundle` después de
  `&[CodexEntry]`. El comando `export_project` pre-resuelve el bundle
  walking docs + llamando a `MediaService::read` antes de despachar
  al renderer.

### Fixed — Editor font realmente aplicado

- El editor traía Lora hardcodeado en CSS, ignorando el setting
  `editor.font` que la UI guardaba. Sprint B lo conecta vía variable
  CSS — los presets viejos siguen funcionando, pero ahora sí
  cambian la tipografía visible.

## [0.9.0-beta] — 2026-05-28

Sprint A cerrado al 100% (10 historias). Primer sprint del backlog v3
con foco interno: deuda técnica acumulada en archivos sobre límite del
CLAUDE.md + accesibilidad base en componentes custom + patrón
premium-ready aplicado al project manager. No hay features nuevas
visibles al usuario; el output es un codebase más mantenible y
navegable con teclado / lector de pantalla, listo para sumar features
en Sprints B/C sin disparar la deuda.

### Changed — Refactors de deuda técnica

- **`storage/documents.rs` (664 líneas) dividido en core + tags +
  positions** (A-01): el archivo queda con CRUD y un helper
  `select_one` reusable; nuevos hermanos `document_tags.rs` (con
  `set` y `list_project_tags`) y `document_positions.rs` (con
  `reorder`). El trait `StorageService` no cambia: el impl en
  `mod.rs` delega a los tres submódulos según la operación.
- **Fix silent error en `documents::create`** (A-04): el `MAX(position)`
  query usaba `.unwrap_or(0)` que silenciaba cualquier error de
  SQLite — incluyendo casos donde un `0` válido pisaría una row
  existente. Ahora `.optional()?.unwrap_or(0)` distingue "tabla vacía"
  (defaultea 0, correcto) de "error real" (propaga al caller).
- **`exporter/docx.rs` (454 líneas) dividido extrayendo
  `docx_helpers.rs`** (A-02): el pipeline HTML→Paragraph
  (`render_html_blocks`, `process_block`, `collect_runs`, `push_runs`,
  `InlineMarks`, `BlockCtx`, `ListKind`) y el builder del Codex
  appendix se mueven al helper. `render()` queda como orquestador
  delgado con `add_title_page`, `add_toc`, `add_document` locales.
- **`services/backup.rs` (461 líneas) extrae `retention_policy.rs`**
  (A-03): la política de pruning (manuals always + top-N dailies +
  monthly anchors) pasa a una función pura `compute_keep_ids` con
  signature `(records, daily_retain, monthly_retain) -> HashSet`,
  testeable sin tempdirs. `prune_old_backups` queda en ~15 líneas
  (delegar + rm_file). 5 tests nuevos para la política.
- **`composables/useWritingTimer.ts` (179 líneas) dividido en 3**
  (A-05): `useWritingTimer` (state machine) + `useBeepAudio`
  (WebAudio API) + `useTimerStorage` (localStorage namespace
  `draffity.timer.*`). El shape público del composable no cambia,
  los 11 tests existentes pasan sin tocar.
- **`composables/useFindReplace.ts` (146 líneas) extrae
  `useProseMirrorSearch.ts`** (A-06): la función `findMatches` ahora
  es pura sobre un `ProseMirrorNode` (doc, query, caseSensitive →
  matches), testeable sin editor TipTap montado. 5 tests nuevos
  cubren empty query, case-sensitive/insensitive, multi-paragraph y
  no-overlap en consecutivos.
- **`ProjectManager` ahora es `trait ProjectManagerService` +
  `LocalProjectManager` impl** (A-10): aplica el patrón premium-ready
  del §2 del CLAUDE.md al manager, que era un struct concreto sin
  interfaz. `AppState.project_manager` pasa a
  `Arc<dyn ProjectManagerService>`; los commands no cambian. Un
  futuro `CloudProjectManager` (sync con backend premium) se enchufa
  sin tocar nada del wiring actual.

### Added — Accesibilidad

- **ARIA labels en componentes custom** (A-07):
  - `EditorToolbar`: cada uno de los 17 botones suma
    `:aria-label` con la misma traducción del tooltip + los toggleable
    (heading/bold/italic/etc.) llevan `:aria-pressed` reflejando
    `isActive`. Wrapper recibe `role="toolbar" :aria-label`.
    Decorativos (separators, icon-overlays como H1, U, S, R, C, el
    icon de blockquote) reciben `aria-hidden="true"`.
  - `FindReplaceBar`: inputs query/replacement suman `:aria-label`
    (no solo placeholder); el counter pasa a region live polite que
    anuncia cambios al cambiar (X/Y matches → no matches → …).
  - `SaveIndicator`: pasa de span pelado a region live polite así el
    estado guardando/guardado/error se anuncia sin invadir.
  - `Binder`: contenedor recibe role navigation con aria-label
    propio.
- **Focus-visible ring global** (A-08): nuevo CSS en
  `styles/main.css` con outline 2px primary + offset 2px sobre
  `*:focus-visible`. PrimeVue components mantienen sus propios
  estilos; este catch-all rescata custom buttons, cards, chips y
  divs interactivos que no tenían foco visible.

### Architecture audit (A-08)

Verificado que PrimeVue 4 da por default:

- Dialogs con `modal` cierran con Esc (`closeOnEscape: true`) y
  atrapan foco; el `OnboardingDialog` tiene `:closable="false"`
  adrede (flujo forzado), no es bug.
- PrimeVue Tree (binder) ya soporta ArrowUp/Down para navegar,
  Enter/Space para seleccionar, ArrowLeft/Right para colapsar.

### Tests

- Rust: 153 verdes (149 lib + 4 integración).
  149 lib: 144 previos + 5 nuevos de `retention_policy`.
- Vitest: 35 verdes (30 previos + 5 nuevos de `useProseMirrorSearch`).
- Playwright: 3 specs sin cambios.

## [0.8.0-beta] — 2026-05-28

Sprint 7 cerrado al 100%: codex (worldbuilding/personajes) llega en
free y diferencia a Draffity de la competencia. La app ahora suma un
catálogo de personajes/lugares/objetos/notas por proyecto, con
cross-references `[[Nombre]]` desde el editor que sobreviven renombrados
(la resolución es por id, no por nombre) y un apéndice opcional en el
export.

### Added — Sprint 7

- **Backend codex** (S7-01..04): migración 009_codex.sql con
  tabla `codex_entries` (columnas id, project_id, kind, name, body,
  tags_json, created_at, updated_at) + FK cascade + tres índices
  (project, project+kind, project+name) para los lookups del picker.
  Dominio
  `CodexEntry` + `CodexKind` (character/place/object/note) +
  `CodexInput` con `validate()` y `normalised()` (trim de nombre,
  dedupe de tags preservando orden first-seen). Storage submodule con
  CRUD + `LIKE` search escapado sobre name/body/tags_json + filtro
  opcional por kind. IPC: `create_codex_entry`, `list_codex_entries`,
  `get_codex_entry`, `update_codex_entry`, `delete_codex_entry`,
  `search_codex_entries`. shared-types añade `CodexEntry`,
  `CodexKind`, `CodexInput`, `CodexUpdate`.
- **Decisión arquitectónica**: codex vive en `StorageService` igual
  que citations, en lugar del trait separado que pedía el backlog. La
  justificación es evitar duplicar conexiones SQLite por puro
  adapter; cuando aparezca un `CodexService` premium (auto-detección
  con IA, sync remoto) se extrae el trait sin tocar consumidoras.
- **Vista Codex como cuarto modo del project view toggle** (S7-05,
  S7-06): `ProjectViewMode` suma `'codex'` al lado de
  editor/corkboard/outliner. `CodexView` arma una grilla responsive
  de `CodexEntryCard` (ícono y color por kind, body preview clampeado
  a 220 caracteres, tags). Header con filtros: search libre por
  name/body/tags + select por kind + select por tag (sourced del
  store). Botón "nueva entrada" abre `CodexEntryDialog` reutilizable
  para crear/editar con name + kind + body (Textarea auto-resize) +
  tags via AutoComplete que sugiere del set de tags ya usados.
  Borrado vía `useConfirm`. Store `useCodexStore` cachea por proyecto
  con helpers `byId`, `byNameLower` (para el cross-ref picker) y
  `allTags`.
- **Cross-refs `[[Nombre]]` en editor** (S7-07, S7-08): nuevo node
  `codexRef` inline-atom con attrs `entryId` + `entryName`. HTML
  serializado: `<span data-codex-ref="<id>">[[Name]]</span>`. El id
  es la fuente de verdad — renombrar `Aragorn` → `Strider` no rompe
  los manuscritos existentes; el name visible es el que quedó al
  insertar. `addProseMirrorPlugins` añade un Plugin con
  `handleClickOn` que despacha un `CustomEvent` `draffity:open-codex`
  con el id en `detail` sobre `window`. `ProjectView` registra el
  listener en
  `onMounted`/`onBeforeUnmount`: al recibirlo cambia view a `'codex'`
  y carga el store si hace falta. Toolbar suma botón "insertar
  referencia codex" (pi-link) que abre `CodexRefPickerDialog`
  (filtro libre por name/body/tags). Estilos del node: subrayado
  punteado + tinte ámbar para distinguirlo de las citations azules.
- **Codex como apéndice opcional en export** (S7-09): nuevo flag
  `ExportConfig.include_codex` (default false). `ExportService::export`
  ahora recibe `&[CodexEntry]` adicional; `export_project` carga
  `storage.list_codex_entries` y se lo pasa. Cada renderer
  (markdown, docx, epub) emite la sección al final cuando el flag
  está activo: heading "Codex" + sub-secciones por kind
  (Characters/Places/Objects/Notes) con name + tags + body HTML.
  EPUB lo agrega como `codex.xhtml` separado; DOCX abre con page
  break antes; Markdown anexa al archivo final. `ExportDialog` suma
  checkbox "incluir codex como apéndice" en la sección Maquetación.

### Tests — Sprint 7

- **S7-10 integración codex** (`tests/codex_integration.rs`, 3 tests):
  - `cross_refs_survive_export_and_codex_appendix_lists_every_entry`:
    crea proyecto + 3 entries + chapter con `[[cross-refs]]`
    embebidas, exporta EPUB + Markdown, verifica que `codex.xhtml`
    está en el zip y que Markdown contiene los nombres de las
    entries y las subsecciones por kind.
  - `codex_appendix_is_skipped_when_include_codex_is_false`: export
    sin el flag no incluye `codex.xhtml`.
  - `rename_keeps_cross_refs_pointing_to_the_same_entry`: rename de
    entry mantiene los cross-refs estables porque la resolución es
    por id, no por nombre — el contrato clave del sprint.
- Rust: 147 verdes (144 lib previas + 12 codex storage + 6 codex
  dominio + 3 integración, repartidas como 144 lib + 4 integración).
- Vitest: 30 verdes.

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

[Unreleased]: https://github.com/OWNER/draffity/compare/v0.9.0-beta...HEAD
[0.9.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.9.0-beta
[0.8.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.8.0-beta
[0.7.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.7.0-beta
[0.6.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.6.0-beta
[0.5.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.5.0-beta
[0.4.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.4.0-beta
[0.3.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.3.0-beta
[0.2.0-beta]: https://github.com/OWNER/draffity/releases/tag/v0.2.0-beta
[0.1.0-alpha]: https://github.com/OWNER/draffity/releases/tag/v0.1.0-alpha
