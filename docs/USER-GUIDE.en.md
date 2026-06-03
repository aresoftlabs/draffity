# Draffity — User guide (beta)

> 100% free app. All content is stored locally on your machine.

## Installation

- **Windows**: download `draffity_<version>_x64_en-US.msi` and run the installer.
- **Linux**: download `draffity_<version>_amd64.AppImage`, make it executable (`chmod +x ...`) and open it.

On first launch you'll see a 3-screen onboarding — follow it or skip it. The final step opens the new-project wizard directly.

## Creating your first project

1. Click **Create new project** on the dashboard.
2. **Step 1 — Template**: choose between `Generic`, `Three-act novel`, `IMRaD paper` or `Shōnen manga`. You'll see a preview of the initial document tree.
3. **Step 2 — Metadata**: enter the title and fill in any extra fields the template requires (author, genre, etc.).
4. **Step 3 — Confirm**: review the summary and click **Create**.

The project opens with the seeded structure on the left-hand binder.

## The active-project rule

Draffity allows **one active project** at a time. Everything else is **archived as read-only**:

- You can keep reading and exporting archived projects.
- You can't edit text in archived projects.
- To edit an archived project, just activate it: the current one gets archived automatically. No data is lost in either direction.

## Editor

- Three panels: **Binder** (structure) · **Editor** (text) · **Inspector** (metadata + versions).
- Auto-save kicks in after you stop typing (configurable in Settings, default 500 ms).
- **Binder drag & drop**: drag chapters and scenes to reorder them or move them between folders. Changes persist when you drop.
- **Focus mode**: hides the binder and inspector so you can write distraction-free. Use the header button or `F11`.
- Shortcuts:
  - `Ctrl+S` — force immediate save
  - `Ctrl+N` — new chapter
  - `Ctrl+B` / `Ctrl+I` / `Ctrl+U` — formatting (bold / italic / underline)
  - `Ctrl+F` — find in the current document
  - `Ctrl+H` — find & replace in the current document
  - `Ctrl+Shift+F` — full-text search across the whole project
  - `F11` — toggle focus mode

## Search

- **In the current document (Ctrl+F)**: bar above the editor with a search field. `Enter` jumps to the next match, `Shift+Enter` to the previous one, `Esc` closes.
- **Replace (Ctrl+H)**: same bar plus a "Replace with…" field and the `Replace` / `Replace all` buttons.
- **Cross-project (Ctrl+Shift+F)**: modal dialog that searches titles and content of every document in the active project, with highlighted snippets. Clicking a result jumps to that document.

## Versions (snapshots)

In the Inspector, **Versions** section:

- Click **+** to save the current version of the document, optionally with a label.
- Hover a version and click the **↺** icon to restore it.
- When you restore, **the previous state is auto-saved** as an `auto-restore` snapshot so you can always go back.

## Export

1. From the project header, click **Export**.
2. Choose a format: `Markdown`, `Word (DOCX)` or `EPUB`.
3. Pick a destination on disk.

The exporter walks the whole document tree in display order.

## Settings

- **Theme** — light, dark or follow OS.
- **Language** — Spanish or English (full UI).
- **Editor font** — serif (Lora), sans (Inter) or monospace (JetBrains Mono).
- **Auto-save interval** — 200 ms to 3 seconds.
- **Writing habit** — current and longest streak (consecutive days).

## AI (optional, BYOK)

Draffity can assist your writing with AI actions (Continue, Expand, Rewrite, Describe) and analytical validators. To enable them, you need your own [OpenRouter](https://openrouter.ai/) API key:

1. Create an account at openrouter.ai and generate an API key.
2. In **Settings → AI & models**, paste the key in the corresponding field.
3. The key is stored in the OS keyring — never in plain text.

Without a key the app is fully functional — AI is an optional add-on.

## Voice (optional, local binaries)

Dictation (Whisper) and read-aloud (Piper) are not included in the base installer. To enable them:

1. In **Settings → Audio & voice**, click **Download model** to install the Whisper model (≈570 MB, with a progress bar and checksum verification).
2. The Piper binary is downloaded alongside the default TTS voice model.
3. Once downloaded, dictation (`Ctrl+Shift+M`) and read-aloud (`Ctrl+Shift+R`) become available.

You can also import binaries manually if you already have the files.

## Data & privacy

- Everything is stored on your local disk: a single SQLite file at `<app data folder>/draffity.db` (on Windows: `%APPDATA%\cl.aresoft.draffity`).
- Logs go to `<app data folder>/logs/draffity.log` with daily rotation.
- No telemetry or external servers.

## Known issues in beta

- PDF export is not yet available — coming in a follow-up iteration.
- macOS is not supported in this beta (planned for v1.0).

## Reporting a bug

Check `<app data folder>/logs/draffity.log` and open an issue in the repository, attaching the log and steps to reproduce.
