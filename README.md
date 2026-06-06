<h1 align="center">Draffity</h1>

<p align="center">
  <strong>The free, local-first writing studio for every format.</strong><br>
  Novels · papers · manga · articles · screenplays — one fast desktop app, yours forever.
</p>

<p align="center">
  <a href="https://github.com/aresoftlabs/draffity/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/aresoftlabs/draffity/actions/workflows/ci.yml/badge.svg"></a>
  <a href="./LICENSE"><img alt="License: GPL v3" src="https://img.shields.io/badge/License-GPLv3-blue.svg"></a>
  <a href="https://draffity.com"><img alt="Version" src="https://img.shields.io/badge/version-0.14.0--beta-7c3aed"></a>
  <a href="https://draffity.com"><img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%C2%B7%20Linux-2563eb"></a>
  <img alt="Free & open source" src="https://img.shields.io/badge/free%20%26%20open%20source-forever-16a34a">
</p>

<p align="center">
  <a href="https://tauri.app"><img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white"></a>
  <a href="https://www.rust-lang.org"><img alt="Rust" src="https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white"></a>
  <a href="https://vuejs.org"><img alt="Vue 3" src="https://img.shields.io/badge/Vue-3-4FC08D?logo=vuedotjs&logoColor=white"></a>
  <a href="https://www.typescriptlang.org"><img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white"></a>
  <img alt="SQLite" src="https://img.shields.io/badge/SQLite-local-003B57?logo=sqlite&logoColor=white">
</p>

<p align="center">
  <strong>English</strong> · <a href="./README.es.md">Español</a> &nbsp;|&nbsp;
  <a href="https://draffity.com"><strong>Download →</strong></a>
</p>

---

Draffity is a fast, lightweight desktop writing studio for long-form work in **any
format** — novels, academic papers, manga, articles, screenplays. It pairs a
**structured binder** and a **worldbuilding codex** with a full rich-text editor,
optional AI assistance (bring your own key), and **100% local voice**. Everything runs
on your machine, offline-first. **No tiers, no subscriptions, no feature gates.**

## Why Draffity

- 🆓 **Free & open source, forever.** GPL-3, no premium plan, no paywalled features, no upsells.
- 🔒 **Local-first & private.** Your manuscripts live in a single file on your disk. No telemetry, no accounts, no servers.
- 🗂️ **A multi-format writing studio.** One tool for the whole spectrum of long-form writing — structured binder, outline and codex included.
- 🤖 **AI on your terms + local voice.** Inline AI with your own OpenRouter key, plus fully on-device dictation and read-aloud.

## Status

**v0.14.0 (beta).** Complete TipTap editor, binder with structured outline views, codex,
DOCX/EPUB/Markdown/PDF export & import, backups, templates, statistics, and accessibility.
Inline AI editing with BYOK (OpenRouter) — continue / expand / rewrite / describe, with
streaming and diff. Local voice (TTS + dictation, including live streaming dictation).
Global language across UI **and** voice in 5 languages. See [`CHANGELOG.md`](./CHANGELOG.md)
for the full history.

The app **updates itself**: Windows and Linux receive signed auto-updates served from
`bins.draffity.com`. macOS and OS-level code signing are on the roadmap.

User guide: [English](./docs/USER-GUIDE.en.md) · [Español](./docs/USER-GUIDE.md).

## Download & install

Get the latest build from **[draffity.com](https://draffity.com)**:

- **Windows** — `*-setup.exe` (NSIS installer; per-user, no admin rights required).
- **Linux** — `*.AppImage` (`chmod +x` and run).

Builds are signed for the in-app updater; OS-level code signing is not in place yet, so
Windows SmartScreen may warn on first launch (_More info → Run anyway_). macOS is coming.
Prefer to build it yourself? See [Development](#development).

## Features

- ✍️ Full rich-text editor (TipTap), multi-format: novel, paper, manga, article, screenplay.
- 🗂️ Structured binder (outline + folders) with a worldbuilding codex.
- 🤖 Inline AI with **BYOK** (OpenRouter): continue / expand / rewrite / describe, with streaming and diff.
- 🎙️ **Local** voice: dictation (Whisper ASR, with live streaming) + read-aloud (Piper TTS), GPU-accelerated.
- 📤 Export / import: DOCX · EPUB · Markdown · PDF (print-ready).
- 🌍 5 languages across UI and voice: English · Español · Français · Italiano · Português.
- 💾 Backups, templates, statistics, accessibility.
- 🆓 100% free, no tiers or subscriptions. Local-first, no telemetry.

## Requirements (to build from source)

- **Node.js** ≥ 20
- **pnpm** ≥ 10
- **Rust** stable (≥ 1.77)
- **Tauri CLI** 2.x (`cargo install tauri-cli --version "^2"`, or comes with `pnpm install`)
- On Linux: WebKit dependencies (see [Tauri prerequisites](https://tauri.app/start/prerequisites/))

## Development

```bash
pnpm install         # install all workspace deps
pnpm tauri:dev       # start Vite + Tauri window with hot reload
```

## Useful scripts

| Script             | Action                                           |
| ------------------ | ------------------------------------------------ |
| `pnpm dev`         | Frontend only (Vite)                             |
| `pnpm tauri:dev`   | Full Tauri + Vue app in dev mode                 |
| `pnpm tauri:build` | Installable build (NSIS Windows, AppImage Linux) |
| `pnpm test`        | Unit tests (Rust + Vitest)                       |
| `pnpm test:e2e`    | E2E with Playwright                              |
| `pnpm lint`        | ESLint + clippy                                  |
| `pnpm fmt`         | Prettier + rustfmt                               |
| `pnpm typecheck`   | `vue-tsc --noEmit`                               |

## Repository layout

```
draffity/
├── apps/
│   ├── desktop/       # Tauri 2 + Rust (backend, IPC, domain)
│   └── ui/            # Vue 3 + TypeScript (frontend)
├── packages/
│   ├── templates/     # Built-in templates (JSON)
│   └── shared-types/  # Shared Rust ↔ TS types
├── docs/              # ARCHITECTURE, WORKFLOW, USER-GUIDE, AUTO-UPDATE, ADR/
└── ...
```

## Architecture

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md). Key patterns:

- **Rust service traits** (`StorageService`, `ExportService`, `AIService`, `ASRService`, `TTSService`) with swappable local implementations — new backends plug in without touching the core.
- **Single canonical SQLite DB** (`~/.draffity/draffity.db`) with additive, versioned migrations. FTS5 over `documents` for cross-project search. The "1 active project" invariant is enforced by a partial UNIQUE index at the SQL level. Reasoning in [ADR 0002](./docs/ADR/0002-sqlite-canonico-vs-por-proyecto.md).
- **Templates as plugins**, described in JSON and discovered under `packages/templates/`.
- **Signed auto-update** served from a public R2 bucket. See [`docs/AUTO-UPDATE.md`](./docs/AUTO-UPDATE.md).

## Product model

Draffity is completely free. **1 active project** (editable, exportable) + **N archived
projects** (read-only). No limits on words, chapters or exports. AI is BYOK (bring your own
OpenRouter key). Voice is local, no subscription.

## Contributing

Contributions are welcome! Read **[CONTRIBUTING](./CONTRIBUTING.md)** and the
**[branch workflow](./docs/WORKFLOW.md)** — branch from `develop`, use Conventional Commits,
and sign off with DCO (`git commit -s`). Be kind: [Code of Conduct](./CODE_OF_CONDUCT.md).
Security reports: [SECURITY](./SECURITY.md). Third-party notices:
[THIRD-PARTY-NOTICES](./THIRD-PARTY-NOTICES.md).

## License

GPL-3.0-or-later. See [`LICENSE`](./LICENSE).

Draffity is maintained by **[Aresoft SpA](mailto:hello@draffity.com)** — a public,
open-source project, 100% free forever, with no premium tier and no direct monetization.
