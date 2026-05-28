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
│   SQLite (per project) | filesystem | (premium: cloud, IA)  │
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

- **Un archivo SQLite por proyecto** (`~/.draffity/projects/<id>/project.db`).
- Migraciones versionadas en `apps/desktop/src/migrations/`.
- Tablas reservadas para premium (no creadas en MVP): `ai_cache`, `sync_state`, `codex_entries`, `voice_takes`.

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

## Por qué Tauri 2 sobre Electron

- Bundles ~5–10× más livianos (target MVP: instalable Win < 15 MB, Linux < 20 MB).
- IPC seguro tipado, sin Node runtime expuesto.
- Permissions/capabilities granulares en `apps/desktop/capabilities/`.
- Backend Rust → robustez en procesamiento de texto, exportadores nativos, performance.

## Decisiones explícitas

- **No Mac en el MVP.** Post-MVP: Apple Developer ID + notarización.
- **No infra propia de IA.** Premium = BYOK. Sin servidores LLM hosted.
- **No integración con publishers** (KDP/D2D/IngramSpark). Si llega, será como export "ready-for-X".
