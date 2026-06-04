# Draffity

> Asistente de escritura desktop multi-formato — novela, paper, manga, artículo, guion.
> **Tauri 2 + Rust + Vue 3 + TypeScript + PrimeVue + Tailwind + TipTap + SQLite.**

[![CI](https://github.com/aresoftlabs/draffity/actions/workflows/ci.yml/badge.svg)](./.github/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE)

Draffity es una aplicación desktop ligera y rápida para escritores que trabajan en distintos formatos. Inspirada en Scrivener, repensada con stack moderno. **100% gratuita y de código abierto**: editor completo, IA con BYOK (OpenRouter), voz local — sin tiers, sin suscripciones, sin gates de funcionalidad.

## Estado

**v0.12.0-beta.** Editor TipTap completo, binder con vistas Scrivener-like, codex, export/import DOCX/EPUB/MD/PDF, backups, plantillas, stats, a11y. Editor IA inline con BYOK (OpenRouter) — continuar/expandir/reescribir/describir con streaming y diff. Voz local (TTS + dictado) integrada. Ver `CHANGELOG.md` para el detalle.

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
├── docs/              # ARCHITECTURE, TEMPLATES-SPEC, ADR/
├── backlog.md         # Backlog del MVP
└── ...
```

## Arquitectura

Ver [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md). Patrones clave:

- **Service traits Rust** (`StorageService`, `ExportService`, `AIService`, `ASRService`, `TTSService`) con implementaciones locales intercambiables — nuevas implementaciones se conectan sin tocar el core.
- **SQLite canónica única** (`<app_data>/draffity.db`) con migraciones versionadas aditivas. FTS5 sobre `documents` para búsqueda cross-proyecto. Invariante "1 proyecto activo" enforzado por índice parcial UNIQUE a nivel SQL. Razonamiento detallado en [ADR 0002](./docs/ADR/0002-sqlite-canonico-vs-por-proyecto.md).
- **Plantillas como plugins** descriptas en JSON, descubiertas en `packages/templates/`.

## Modelo del producto

Draffity es completamente gratuito. 1 proyecto activo (editable, exportable) + N proyectos archivados (read-only). Sin límite de palabras, capítulos ni exportaciones. IA con BYOK (trae tu propia key de OpenRouter). Voz local sin suscripción.

## Licencia

GPL-3.0-or-later. Ver [`LICENSE`](./LICENSE).

Draffity es mantenido por **[Aresoft SpA](mailto:hola@aresoft.cl)** — proyecto open-source público, 100% gratuito para siempre, sin premium, sin monetización directa.
