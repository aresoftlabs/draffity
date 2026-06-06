# Privacy Policy — Draffity

_Last updated: 2026-05-28._

Draffity is a desktop writing app, designed around one simple
principle: **your text belongs to you and stays on your computer**.

## What Draffity stores

Everything you type lives **only on your machine** inside the OS
application data directory:

- Manuscripts, chapters, scenes and notes (`draffity.db` file).
- Per-document snapshot history.
- Images and fonts you upload to a project (`media/` folder).
- Automatic daily and monthly backups (`backups/` folder).
- Your editor configuration (theme, language, font, shortcuts).
- Local writing stats (streaks, words per day).

No data is uploaded to any remote server. There is no account, no
cloud sync, no analytics.

## What Draffity does NOT collect

- **Usage telemetry**: we don't record what buttons you click or how
  many times you open the app.
- **Crash reports**: off by default. If you explicitly enable them
  under Settings → Privacy, Draffity sends the stack trace and the
  app version to the URL configured at build time (a Sentry instance
  the binary owner controls). The content of your documents is
  never included.
- **Identifiers**: no user UUID, no device fingerprint.

## Third-party components

Draffity uses these libraries that may touch the network only when
you invoke them:

- **Tauri Shell**: open exported files with your default browser or
  editor.
- **Tauri Dialog**: native file pickers.
- **Tauri FS**: read/write files at paths you pick.

None of them transmit data without a deliberate action from you.

## Your right to delete

To remove all Draffity data, delete the `~/.draffity/` folder (on
Windows: `%USERPROFILE%\.draffity\`). The binary uninstaller does
not touch that folder so upgrades don't erase your work.

## Changes to this policy

If this policy changes, the "Last updated" date above is bumped and
the release CHANGELOG mentions the change.

## Data controller

The data controller is **Aresoft SpA** (hello@draffity.com), the
organisation that owns and maintains Draffity.

## Contact

Bugs and privacy questions: open an issue on the Draffity repository
or write to [hello@draffity.com](mailto:hello@draffity.com).

---

[Versión en español](./PRIVACY-POLICY.md)
