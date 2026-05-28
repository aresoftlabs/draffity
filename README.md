# Draffity

> Asistente de escritura desktop multi-formato — novela, paper, manga, artículo, guion.
> **Tauri 2 + Rust + Vue 3 + TypeScript + PrimeVue + Tailwind + TipTap + SQLite.**

[![CI](https://github.com/OWNER/draffity/actions/workflows/ci.yml/badge.svg)](./.github/workflows/ci.yml)

Draffity es una aplicación desktop ligera y rápida para escritores que trabajan en distintos formatos. Inspirada en Scrivener, repensada con stack moderno y un modelo freemium honesto: el tier gratuito es **completamente funcional**; el premium es un complemento (multi-proyecto, IA con BYOK, cloud sync, voz a texto), nunca un gate de funcionalidad básica.

## Estado

**v0.2.0-beta — Sprint 1 cerrado.** Drag&drop en binder, búsqueda full-text por proyecto (FTS5), find & replace en documento, modo enfoque (F11), onboarding integrado con creación de proyecto. Ver [`backlog.md`](./backlog.md) y `CHANGELOG.md` para el detalle.

Guía de usuario: [ES](./docs/USER-GUIDE.md) · [EN](./docs/USER-GUIDE.en.md).

## Requisitos

- **Node.js** ≥ 20
- **pnpm** ≥ 10
- **Rust** estable (≥ 1.77)
- **Tauri CLI** 2.x (`cargo install tauri-cli --version "^2"` o vendrá con `pnpm install`)
- En Linux: dependencias de WebKit (ver [docs Tauri prerequisites](https://tauri.app/start/prerequisites/))

## Desarrollo

```bash
pnpm install         # instala todas las deps del workspace
pnpm tauri:dev       # arranca Vite + ventana Tauri en hot reload
```

## Scripts útiles

| Script             | Acción                                         |
| ------------------ | ---------------------------------------------- |
| `pnpm dev`         | Solo el frontend Vue (Vite)                    |
| `pnpm tauri:dev`   | App completa Tauri + Vue en modo dev           |
| `pnpm tauri:build` | Build instalable (MSI Windows, AppImage Linux) |
| `pnpm test`        | Tests unit Rust + Vitest                       |
| `pnpm test:e2e`    | E2E con Playwright                             |
| `pnpm lint`        | ESLint + clippy                                |
| `pnpm fmt`         | Prettier + rustfmt                             |
| `pnpm typecheck`   | `vue-tsc --noEmit`                             |

## Estructura del repo

```
draffity/
├── apps/
│   ├── desktop/       # Tauri 2 + Rust (backend, IPC, dominio)
│   └── ui/            # Vue 3 + TypeScript (frontend)
├── packages/
│   ├── templates/     # Plantillas built-in (JSON)
│   └── shared-types/  # Tipos compartidos Rust ↔ TS
├── docs/              # ARCHITECTURE, PREMIUM-INTEGRATION, TEMPLATES-SPEC
├── backlog.md         # Backlog del MVP
└── ...
```

## Arquitectura

Ver [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md). Patrones clave:

- **Service traits Rust** (`StorageService`, `ExportService`, `AIService`, `CloudSyncService`, `ASRService`) con implementaciones NoOp en MVP — premium se añade como nuevas implementaciones, sin tocar el core.
- **Capability gates centralizados** en `apps/desktop/src/capabilities.rs`. La UI consulta vía `useCapability()`.
- **SQLite canónica única** (`<app_data>/draffity.db`) con migraciones versionadas. Tablas premium reservadas para migraciones futuras. FTS5 sobre `documents` para búsqueda cross-proyecto. Razonamiento detallado en [ADR 0002](./docs/ADR/0002-sqlite-canonico-vs-por-proyecto.md).
- **Plantillas como plugins** descriptas en JSON, descubiertas en `packages/templates/`.

## Modelo del producto

- **Free**: 1 proyecto activo (editable, exportable) + N proyectos archivados (read-only). Sin límite de palabras, capítulos ni exportaciones.
- **Premium** (post-MVP): multi-proyecto activo, IA con BYOK (OpenRouter), cloud sync/backup, voz a texto, plantillas premium.

## Licencia

MIT. Ver [`LICENSE`](./LICENSE).
