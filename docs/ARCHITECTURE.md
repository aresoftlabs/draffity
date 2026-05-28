# Architecture

> Estado: Fase 0 (scaffolding). Documento vivo — se actualiza al final de cada épica.

## Visión global

```
┌──────────────────── Vue 3 SPA (apps/ui) ────────────────────┐
│  views/  components/  editor/  composables/  stores/  ...   │
│                       │                                     │
│           services/ipc.ts (typed IPC client)                │
└───────────────────────┬─────────────────────────────────────┘
                        │  Tauri IPC (JSON, typed)
┌───────────────────────▼─────────────────────────────────────┐
│            commands/  (apps/desktop/src/commands)           │
│                       │                                     │
│   services/  (StorageService, ExportService, AIService,     │
│   CloudSyncService, ASRService, TierService — traits)       │
│                       │                                     │
│   domain/   (Project, Document, Snapshot, Template, Tier)   │
│                       │                                     │
│   SQLite (single canonical DB) | filesystem | (premium: cloud, IA)  │
└─────────────────────────────────────────────────────────────┘
```

## Capas

### `domain/` — pura

Entidades, invariantes, value objects. No conoce SQLite ni Tauri. Testeable aislado. Aquí vive la regla "1 proyecto activo + archivado read-only".

### `services/` — efectos

Traits + implementaciones. En el MVP sólo existen las versiones locales/NoOp. Premium se añade implementando los mismos traits con backends remotos.

| Trait              | MVP impl                       | Premium impl                 |
| ------------------ | ------------------------------ | ---------------------------- |
| `StorageService`   | `LocalStorageService` (SQLite) | `CloudSyncStorageService`    |
| `ExportService`    | `LocalExporter` (DOCX/EPUB/MD) | + plantillas premium         |
| `AIService`        | `NoOpAI`                       | `OpenRouterAIService` (BYOK) |
| `CloudSyncService` | `NoOpSync`                     | `S3Sync` / `R2Sync`          |
| `ASRService`       | `NoOpASR`                      | `WhisperLocalASR`            |
| `TierService`      | `FreeTier`                     | `PremiumTier`                |

### `commands/` — IPC

Cada comando Tauri es una función `#[tauri::command]` fina que invoca servicios. Cero lógica de negocio aquí.

### `capabilities.rs` — feature gates

Tabla central que mapea `Tier → HashSet<Capability>`. La UI nunca decide capabilities; sólo consulta via `capability_enabled` IPC.

## Storage

- **Un único archivo SQLite canónico** en `<app_data_dir>/draffity.db` (Windows: `%APPDATA%\cl.aresoft.draffity\draffity.db`). Ver [ADR 0002](./ADR/0002-sqlite-canonico-vs-por-proyecto.md) para el razonamiento.
- Modo WAL + `foreign_keys=ON` + `synchronous=NORMAL`.
- Migraciones versionadas en `apps/desktop/src/migrations/`, aplicadas en orden por `LocalStorageService::migrate()`.
- Range `100_*` reservado para migraciones premium (no aplicadas en free).
- Tablas reservadas para premium: `ai_cache`, `sync_state`, `codex_entries`, `voice_takes`.

## Frontend

- **Vue 3 Composition API + `<script setup>`** en todos los componentes.
- **Pinia** stores: `project`, `document`, `ui`. Solo estado UI/cliente — la verdad vive en Rust/SQLite.
- **TipTap** como editor (extensible vía plugins; premium añadirá extensions sin tocar el core).
- **PrimeVue + Tailwind**: componentes ricos (Tree, Splitter, Dialog) + utility CSS.
- **vue-i18n** ES/EN desde día uno.

## Event bus

Rust emite eventos con `tauri::Manager::emit`:

- `project.opened`, `project.archived`, `project.created`, `project.deleted`
- `document.created`, `document.saved`, `document.deleted`, `document.moved`
- `snapshot.created`, `snapshot.restored`

La UI se suscribe con `listen()`. **Premium** (cloud sync, AI background) puede suscribirse igual sin tocar el core.

## Patrones canónicos

Estos cinco patrones aparecen repetidos en el código y son **el lenguaje arquitectónico del proyecto**. Toda feature nueva debe encajar en alguno o justificar explícitamente por qué inventa uno nuevo.

### 1. Trait + impl NoOp (services premium-ready)

Cualquier capa que pueda tener variante premium se define como `trait` con impl local (free) y `NoOp` cuando aplique. Premium = nueva impl del mismo trait, sin tocar nada existente.

**Ejemplos en el código**: [`AIService`](../apps/desktop/src/services/ai.rs), [`CloudSyncService`](../apps/desktop/src/services/sync.rs), [`ASRService`](../apps/desktop/src/services/asr.rs), [`TierService`](../apps/desktop/src/services/tier.rs). Ver [ADR 0003](./ADR/0003-premium-aditivo-via-traits.md).

### 2. Strategy (export, futuro: import)

Cada formato es struct independiente que implementa una operación común. El orquestador despacha por `HashMap<Format, Box<dyn Strategy>>`, **no** por `match format { ... }` inline.

**Ejemplos**: [`exporter/markdown.rs`](../apps/desktop/src/services/exporter/markdown.rs), [`exporter/docx.rs`](../apps/desktop/src/services/exporter/docx.rs), [`exporter/epub.rs`](../apps/desktop/src/services/exporter/epub.rs), orquestados desde [`exporter/mod.rs`](../apps/desktop/src/services/exporter/mod.rs).

### 3. Capability gate central

Único lugar de verdad: [`capabilities.rs`](../apps/desktop/src/capabilities.rs). UI consulta vía `useCapability('name')`. Backend vía `tier.is_enabled("name")`. **Cero chequeos de tier inline** en lógica de negocio.

### 4. Atomic transaction multi-paso

Operaciones que tocan ≥2 tablas o ≥2 rows van en `conn.transaction()`. Un fallo intermedio nunca debe dejar estado inconsistente.

**Ejemplos**: [`create_project_atomic`](../apps/desktop/src/services/storage.rs), [`restore_snapshot`](../apps/desktop/src/services/storage.rs) (que crea auto-snapshot antes de sobrescribir).

### 5. Event bus para desacoplar suscriptores

Si feature B necesita reaccionar a algo que hace feature A, **A emite evento, B se suscribe**. No imports directos cruzados. Esto es lo que permite que cloud sync, AI background y otras features premium se enganchen sin tocar el core.

**Ejemplos**: [`events.rs`](../apps/desktop/src/events.rs) emite `project.opened`, `document.saved`, `snapshot.created`, etc.

## Decisiones registradas (ADRs)

Las decisiones arquitectónicas importantes se documentan en [`docs/ADR/`](./ADR/):

- [ADR 0001 — Tauri 2 sobre Electron](./ADR/0001-tauri-sobre-electron.md)
- [ADR 0002 — SQLite canónica única vs por proyecto](./ADR/0002-sqlite-canonico-vs-por-proyecto.md)
- [ADR 0003 — Premium aditivo vía traits](./ADR/0003-premium-aditivo-via-traits.md)

## Decisiones explícitas (alcance del MVP)

- **No Mac en el MVP.** Llega en v1.0.0 (ver `backlog`). Requiere Apple Developer ID + notarización.
- **No infra propia de IA.** Premium = BYOK. Sin servidores LLM hosted.
- **No integración con publishers** (KDP/D2D/IngramSpark). Si llega, será como export "ready-for-X".
