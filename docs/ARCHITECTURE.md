# Architecture

> Estado: v0.14.0 (beta) — editor, binder, codex, export/import, IA (BYOK), voz local
> y auto-update operativos. Documento vivo — se actualiza al final de cada épica.

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
│   ASRService, TTSService — traits + impls locales)          │
│                       │                                     │
│   domain/   (Project, Document, Snapshot, Template)         │
│                       │                                     │
│   SQLite (single canonical DB) | filesystem | sidecars      │
└─────────────────────────────────────────────────────────────┘
```

## Capas

### `domain/` — pura

Entidades, invariantes, value objects. No conoce SQLite ni Tauri. Testeable aislado. Aquí vive la regla "1 proyecto activo + archivado read-only".

### `services/` — efectos

Traits + implementaciones. El patrón `trait + impl local` permite intercambiar backends sin tocar el core; también facilita testing (NoOp en tests unitarios).

| Trait            | Impl principal                 | Notas                                     |
| ---------------- | ------------------------------ | ----------------------------------------- |
| `StorageService` | `LocalStorageService` (SQLite) | única fuente de verdad en disco           |
| `ExportService`  | `LocalExporter` (DOCX/EPUB/MD) | strategy por formato                      |
| `AIService`      | `OpenRouterAIService`          | BYOK: clave OpenRouter del usuario        |
| `ASRService`     | `WhisperLocalASR`              | sidecar Whisper en la máquina del usuario |
| `TTSService`     | `PiperLocalTTS`                | sidecar Piper en la máquina del usuario   |

### `commands/` — IPC

Cada comando Tauri es una función `#[tauri::command]` fina que invoca servicios. Cero lógica de negocio aquí.

## Storage

- **Un único archivo SQLite canónico** en `~/.draffity/draffity.db` (en Windows, `%USERPROFILE%\.draffity\draffity.db`). El root `~/.draffity/` (resuelto por `DraffityHome`) agrupa DB, `voice/`, `media/`, `backups/`, `logs/` y `templates/`, y admite override vía `config.json`. Ver [ADR 0002](./ADR/0002-sqlite-canonico-vs-por-proyecto.md) para el razonamiento.
- Modo WAL + `foreign_keys=ON` + `synchronous=NORMAL`.
- Migraciones versionadas en `apps/desktop/src/migrations/`, aplicadas en orden por `LocalStorageService::migrate()`.

## Frontend

- **Vue 3 Composition API + `<script setup>`** en todos los componentes.
- **Pinia** stores: `project`, `document`, `ui`. Solo estado UI/cliente — la verdad vive en Rust/SQLite.
- **TipTap** como editor (extensible vía plugins de la comunidad y propios).
- **PrimeVue + Tailwind**: componentes ricos (Tree, Splitter, Dialog) + utility CSS.
- **vue-i18n** con idioma global en 5 lenguas (English, Español, Français, Italiano, Português), aplicado a UI **y** voz.

## Event bus

Rust emite eventos con `tauri::Manager::emit`. Los nombres usan `:` como separador
(Tauri 2.11 rechaza `.` en nombres de evento), definidos como constantes en
[`events.rs`](../apps/desktop/src/events.rs):

- `project:created`, `project:opened`, `project:archived`, `project:deleted`
- `document:created`, `document:saved`, `document:moved`, `document:deleted`
- `snapshot:created`

La UI se suscribe con `listen()`. Servicios de fondo (IA, ASR, TTS) pueden suscribirse igual sin tocar el core.

## Patrones canónicos

Estos cuatro patrones aparecen repetidos en el código y son **el lenguaje arquitectónico del proyecto**. Toda feature nueva debe encajar en alguno o justificar explícitamente por qué inventa uno nuevo.

### 1. Trait + impl local

Cualquier capa intercambiable se define como `trait` con impl local y `NoOp` en tests cuando aplique. Añadir un backend alternativo = nueva impl del mismo trait, sin tocar nada existente.

**Ejemplos en el código**: [`AIService`](../apps/desktop/src/services/ai.rs), [`ASRService`](../apps/desktop/src/services/asr.rs), [`TTSService`](../apps/desktop/src/services/tts.rs).

### 2. Strategy (export, import)

Cada formato vive en su propio módulo con una operación común (`render`/`parse`). El conjunto de formatos es **cerrado** (un `enum` exhaustivo: `ExportFormat`, `ImportFormat`), por lo que el orquestador despacha con `match format { ... }` a la función del módulo correspondiente — exhaustividad verificada en compile-time, sin dispatch dinámico. El patrón `HashMap<Format, Box<dyn Strategy>>` queda reservado para un eventual set abierto/extensible (plugins de terceros, registro en runtime). Ver [ADR-0004](./ADR/0004-dispatch-por-match-para-formatos-cerrados.md).

**Ejemplos**: [`exporter/markdown.rs`](../apps/desktop/src/services/exporter/markdown.rs), [`exporter/docx.rs`](../apps/desktop/src/services/exporter/docx.rs), [`exporter/epub.rs`](../apps/desktop/src/services/exporter/epub.rs), orquestados desde [`exporter/mod.rs`](../apps/desktop/src/services/exporter/mod.rs).

### 3. Atomic transaction multi-paso

Operaciones que tocan ≥2 tablas o ≥2 rows van en `conn.transaction()`. Un fallo intermedio nunca debe dejar estado inconsistente.

**Ejemplos**: [`create_project_atomic`](../apps/desktop/src/services/storage.rs), [`restore_snapshot`](../apps/desktop/src/services/storage.rs) (que crea auto-snapshot antes de sobrescribir).

### 4. Event bus para desacoplar suscriptores

Si feature B necesita reaccionar a algo que hace feature A, **A emite evento, B se suscribe**. No imports directos cruzados. Esto es lo que permite que servicios de IA, ASR o TTS se enganchen sin tocar el core.

**Ejemplos**: [`events.rs`](../apps/desktop/src/events.rs) emite `project:opened`, `document:saved`, `snapshot:created`, etc.

## Distribución y auto-update

- **Releases públicas servidas desde R2.** Un tag `v*` dispara `release.yml`, que buildea y firma (minisign) los instaladores (NSIS Windows + AppImage Linux) y los sube a un bucket Cloudflare R2 expuesto vía dominio propio `bins.draffity.com`.
- **Manifiesto propio.** `scripts/build-update-manifest.mjs` arma `app/stable/latest.json`; la app lo lee sin auth y verifica la firma del artefacto contra la `pubkey` en `tauri.conf.json`.
- **Updater in-app.** `tauri-plugin-updater` aplica el update sin prompt de UAC (NSIS per-user en Windows, reemplazo in-place del AppImage en Linux). Detalle completo en [`AUTO-UPDATE.md`](./AUTO-UPDATE.md).

## Decisiones registradas (ADRs)

Las decisiones arquitectónicas importantes se documentan en [`docs/ADR/`](./ADR/):

- [ADR 0001 — Tauri 2 sobre Electron](./ADR/0001-tauri-sobre-electron.md)
- [ADR 0002 — SQLite canónica única vs por proyecto](./ADR/0002-sqlite-canonico-vs-por-proyecto.md)
- [ADR 0003 — Diseño de servicios vía traits](./ADR/0003-service-traits-pattern.md)
- [ADR 0006 — Draffity 100 % gratis](./ADR/0006-draffity-100-gratis.md)

## Decisiones explícitas (alcance del MVP)

- **No Mac en el MVP.** Llega en v1.0.0 (ver `backlog`). Requiere Apple Developer ID + notarización.
- **No infra propia de IA.** La IA usa BYOK: el usuario aporta su clave de OpenRouter. Sin servidores LLM hosted.
- **No integración con publishers** (KDP/D2D/IngramSpark). Si llega, será como export "ready-for-X".
