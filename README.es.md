<h1 align="center">Draffity</h1>

<p align="center">
  <strong>El estudio de escritura local-first y gratuito para todos los formatos.</strong><br>
  Novelas · papers · manga · artículos · guiones — una app de escritorio rápida, tuya para siempre.
</p>

<p align="center">
  <a href="https://github.com/aresoftlabs/draffity/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/aresoftlabs/draffity/actions/workflows/ci.yml/badge.svg"></a>
  <a href="./LICENSE"><img alt="Licencia: GPL v3" src="https://img.shields.io/badge/License-GPLv3-blue.svg"></a>
  <a href="https://draffity.com"><img alt="Versión" src="https://img.shields.io/badge/version-0.14.0--beta-7c3aed"></a>
  <a href="https://draffity.com"><img alt="Plataformas" src="https://img.shields.io/badge/platforms-Windows%20%C2%B7%20Linux-2563eb"></a>
  <img alt="Gratis y open source" src="https://img.shields.io/badge/free%20%26%20open%20source-forever-16a34a">
</p>

<p align="center">
  <a href="https://tauri.app"><img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white"></a>
  <a href="https://www.rust-lang.org"><img alt="Rust" src="https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white"></a>
  <a href="https://vuejs.org"><img alt="Vue 3" src="https://img.shields.io/badge/Vue-3-4FC08D?logo=vuedotjs&logoColor=white"></a>
  <a href="https://www.typescriptlang.org"><img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white"></a>
  <img alt="SQLite" src="https://img.shields.io/badge/SQLite-local-003B57?logo=sqlite&logoColor=white">
</p>

<p align="center">
  <a href="./README.md">English</a> · <strong>Español</strong> &nbsp;|&nbsp;
  <a href="https://draffity.com"><strong>Descargar →</strong></a>
</p>

---

Draffity es un estudio de escritura de escritorio, liviano y rápido, para trabajo de formato
largo en **cualquier formato** — novelas, papers académicos, manga, artículos, guiones.
Combina un **binder estructurado** y un **codex de worldbuilding** con un editor de texto
enriquecido completo, asistencia de IA opcional (con tu propia clave) y **voz 100% local**.
Todo corre en tu máquina, offline-first. **Sin tiers, sin suscripciones, sin gates de
funcionalidad.**

## Por qué Draffity

- 🆓 **Gratis y open source, para siempre.** GPL-3, sin plan premium, sin funciones bajo paywall, sin upsells.
- 🔒 **Local-first y privado.** Tus manuscritos viven en un único archivo en tu disco. Sin telemetría, sin cuentas, sin servidores.
- 🗂️ **Un estudio multi-formato.** Una sola herramienta para todo el espectro de la escritura larga — binder estructurado, outline y codex incluidos.
- 🤖 **IA en tus términos + voz local.** IA inline con tu propia clave de OpenRouter, más dictado y lectura en voz alta totalmente en el dispositivo.

## Estado

**v0.14.0 (beta).** Editor TipTap completo, binder con vistas de outline estructurado, codex,
export/import DOCX/EPUB/Markdown/PDF, backups, plantillas, estadísticas y accesibilidad.
Edición IA inline con BYOK (OpenRouter) — continuar / expandir / reescribir / describir, con
streaming y diff. Voz local (TTS + dictado, incluido dictado en vivo por streaming). Idioma
global en UI **y** voz, en 5 idiomas. Ver [`CHANGELOG.md`](./CHANGELOG.md) para el detalle.

La app **se actualiza sola**: Windows y Linux reciben auto-updates firmados servidos desde
`bins.draffity.com`. macOS y la firma de código a nivel OS están en el roadmap.

Guía de usuario: [Español](./docs/USER-GUIDE.md) · [English](./docs/USER-GUIDE.en.md).

## Descarga e instalación

Descargá el build más reciente desde **[draffity.com](https://draffity.com)**:

- **Windows** — `*-setup.exe` (instalador NSIS; per-user, sin permisos de admin).
- **Linux** — `*.AppImage` (`chmod +x` y ejecutar).

Los binarios están firmados para el updater interno; la firma de código a nivel OS todavía no
está, así que SmartScreen de Windows puede avisar en la primera ejecución (_Más información →
Ejecutar de todos modos_). macOS llega pronto. ¿Preferís compilarlo vos? Ver
[Desarrollo](#desarrollo).

## Características

- ✍️ Editor de texto enriquecido completo (TipTap), multi-formato: novela, paper, manga, artículo, guion.
- 🗂️ Binder estructurado (outline + carpetas) con codex de worldbuilding.
- 🤖 IA inline con **BYOK** (OpenRouter): continuar / expandir / reescribir / describir, con streaming y diff.
- 🎙️ Voz **local**: dictado (Whisper ASR, con streaming en vivo) + lectura (Piper TTS), con aceleración por GPU.
- 📤 Export / import: DOCX · EPUB · Markdown · PDF (listo para imprimir).
- 🌍 5 idiomas en UI y voz: English · Español · Français · Italiano · Português.
- 💾 Backups, plantillas, estadísticas, accesibilidad.
- 🆓 100% gratis, sin tiers ni suscripciones. Local-first, sin telemetría.

## Requisitos (para compilar desde el código)

- **Node.js** ≥ 20
- **pnpm** ≥ 10
- **Rust** estable (≥ 1.77)
- **Tauri CLI** 2.x (`cargo install tauri-cli --version "^2"`, o viene con `pnpm install`)
- En Linux: dependencias de WebKit (ver [prerequisites de Tauri](https://tauri.app/start/prerequisites/))

## Desarrollo

```bash
pnpm install         # instala todas las deps del workspace
pnpm tauri:dev       # arranca Vite + ventana Tauri en hot reload
```

## Scripts útiles

| Script             | Acción                                          |
| ------------------ | ----------------------------------------------- |
| `pnpm dev`         | Solo el frontend (Vite)                         |
| `pnpm tauri:dev`   | App completa Tauri + Vue en modo dev            |
| `pnpm tauri:build` | Build instalable (NSIS Windows, AppImage Linux) |
| `pnpm test`        | Tests unitarios (Rust + Vitest)                 |
| `pnpm test:e2e`    | E2E con Playwright                              |
| `pnpm lint`        | ESLint + clippy                                 |
| `pnpm fmt`         | Prettier + rustfmt                              |
| `pnpm typecheck`   | `vue-tsc --noEmit`                              |

## Estructura del repo

```
draffity/
├── apps/
│   ├── desktop/       # Tauri 2 + Rust (backend, IPC, dominio)
│   └── ui/            # Vue 3 + TypeScript (frontend)
├── packages/
│   ├── templates/     # Plantillas built-in (JSON)
│   └── shared-types/  # Tipos compartidos Rust ↔ TS
├── docs/              # ARCHITECTURE, WORKFLOW, USER-GUIDE, AUTO-UPDATE, ADR/
└── ...
```

## Arquitectura

Ver [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md). Patrones clave:

- **Service traits Rust** (`StorageService`, `ExportService`, `AIService`, `ASRService`, `TTSService`) con implementaciones locales intercambiables — nuevos backends se conectan sin tocar el core.
- **SQLite canónica única** (`~/.draffity/draffity.db`) con migraciones versionadas aditivas. FTS5 sobre `documents` para búsqueda cross-proyecto. La invariante "1 proyecto activo" se enforza con un índice parcial UNIQUE a nivel SQL. Razonamiento en [ADR 0002](./docs/ADR/0002-sqlite-canonico-vs-por-proyecto.md).
- **Plantillas como plugins**, descriptas en JSON y descubiertas en `packages/templates/`.
- **Auto-update firmado** servido desde un bucket R2 público. Ver [`docs/AUTO-UPDATE.md`](./docs/AUTO-UPDATE.md).

## Modelo del producto

Draffity es completamente gratuito. **1 proyecto activo** (editable, exportable) + **N
proyectos archivados** (read-only). Sin límite de palabras, capítulos ni exportaciones. IA con
BYOK (trae tu propia clave de OpenRouter). Voz local, sin suscripción.

## Contribuir

¡Bienvenidas las contribuciones! Leé **[CONTRIBUTING](./CONTRIBUTING.md)** y el
**[workflow de ramas](./docs/WORKFLOW.md)** — branch desde `develop`, Conventional Commits y
DCO (`git commit -s`). Convivencia: [Código de Conducta](./CODE_OF_CONDUCT.md). Reporte de
seguridad: [SECURITY](./SECURITY.md). Avisos de terceros:
[THIRD-PARTY-NOTICES](./THIRD-PARTY-NOTICES.md).

## Licencia

GPL-3.0-or-later. Ver [`LICENSE`](./LICENSE).

Draffity es mantenido por **[Aresoft SpA](mailto:hello@draffity.com)** — un proyecto
open-source público, 100% gratuito para siempre, sin premium y sin monetización directa.
