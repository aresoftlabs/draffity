# Auto-update (Tauri 2 + R2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automatic updates to the Draffity desktop app (Windows NSIS + Linux AppImage), serving signed installers and an update manifest from Cloudflare R2 (`bins.draffity.com`).

**Architecture:** Each `v*` tag triggers `release.yml`, which builds + signs (minisign) the installers, uploads them to `app/<version>/` in R2, and a follow-up job assembles `app/stable/latest.json`. The app polls that manifest at startup (and via a manual button in Settings), verifies the downloaded installer's signature against an embedded public key, and applies the update on user confirmation (NSIS per-user → no UAC).

**Tech Stack:** Tauri 2, `tauri-plugin-updater`, `tauri-plugin-process`, Rust, Vue 3 + Pinia + vue-i18n, Node ≥20 (manifest script + `node --test`), GitHub Actions, AWS CLI → R2.

**Spec:** [`docs/specs/2026-06-05-auto-update-tauri-design.md`](../specs/2026-06-05-auto-update-tauri-design.md)

---

## File Structure

**Create:**

- `scripts/build-update-manifest.mjs` — pure `buildManifest()` + `fragment`/`assemble` CLI subcommands.
- `scripts/build-update-manifest.test.mjs` — `node --test` unit + integration tests.
- `apps/ui/src/composables/useUpdater.ts` — singleton composable: state machine over the updater/process plugins.
- `apps/ui/src/composables/useUpdater.test.ts` — vitest, plugins mocked.
- `apps/ui/src/components/UpdateBanner.vue` — non-intrusive "update available" banner.
- `apps/ui/src/components/UpdateBanner.test.ts` — vitest component test.
- `apps/ui/src/components/SettingsUpdates.vue` — current version + manual "check for updates".

**Modify:**

- `apps/desktop/Cargo.toml` — add updater + process crates.
- `apps/desktop/src/lib.rs` — register the two plugins.
- `apps/desktop/tauri.conf.json` — NSIS target, `createUpdaterArtifacts`, `plugins.updater`.
- `apps/desktop/capabilities/default.json` — `updater:default`, `process:allow-restart`.
- `apps/ui/package.json` — add `@tauri-apps/plugin-updater`, `@tauri-apps/plugin-process`.
- `apps/ui/src/App.vue` — mount `UpdateBanner`, fire silent startup check.
- `apps/ui/src/views/Settings.vue` — mount `SettingsUpdates` in the "about" section.
- `apps/ui/src/locales/es.json`, `apps/ui/src/locales/en.json` — `updater.*` strings.
- `.github/workflows/release.yml` — signing env, R2 upload, fragment artifacts, manifest job.
- `package.json` (root) — `test:scripts` script.
- `docs/AUTO-UPDATE.md` — flip status to "implemented".

---

## Task 1: Generate signing keypair + configure CI secrets (manual prerequisite)

This task changes no repo files. It produces (a) a **public key** pasted into `tauri.conf.json` in Task 4 and (b) two **GitHub secrets** read by `release.yml` in Task 8. Do it once.

- [ ] **Step 1: Generate the minisign keypair**

Run (from repo root; pick any password and remember it):

```bash
pnpm --filter @draffity/desktop tauri signer generate -- -w "$HOME/.draffity/updater.key"
```

Expected: prints a block containing `Public key: <BASE64>` and writes the password-protected private key to `~/.draffity/updater.key`. Copy the **public key** string — it goes into Task 4.

- [ ] **Step 2: Store the private key + password as GitHub secrets**

Run (replace `<password>` with the one chosen in Step 1):

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < "$HOME/.draffity/updater.key" --repo aresoftlabs/draffity
printf '%s' '<password>' | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo aresoftlabs/draffity
```

- [ ] **Step 3: Verify the secrets exist**

Run: `gh secret list --repo aresoftlabs/draffity`
Expected: the list includes `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (plus the pre-existing `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_ENDPOINT`).

- [ ] **Step 4: Record the public key**

Paste the public key from Step 1 into a scratch note; Task 4 Step 3 pastes it into `tauri.conf.json`. The private key file (`~/.draffity/updater.key`) is **never** committed.

> No commit in this task — nothing in the repo changed.

---

## Task 2: Manifest builder — `buildManifest()` pure function (TDD)

**Files:**

- Create: `scripts/build-update-manifest.mjs`
- Test: `scripts/build-update-manifest.test.mjs`
- Modify: `package.json` (root)

- [ ] **Step 1: Add the `test:scripts` runner to root `package.json`**

In `package.json`, add this line to the `"scripts"` object (after `"test:e2e"`):

```json
    "test:scripts": "node --test scripts/",
```

- [ ] **Step 2: Write the failing test**

Create `scripts/build-update-manifest.test.mjs`:

```js
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildManifest } from './build-update-manifest.mjs';

test('buildManifest assembles version, notes, pub_date and platforms map', () => {
  const manifest = buildManifest({
    version: '0.13.0',
    notes: 'Mejoras de exportación.',
    pubDate: '2026-06-05T12:00:00Z',
    platforms: [
      {
        key: 'windows-x86_64',
        url: 'https://bins.draffity.com/app/0.13.0/X_x64-setup.exe',
        signature: 'SIGWIN',
      },
      {
        key: 'linux-x86_64',
        url: 'https://bins.draffity.com/app/0.13.0/X_amd64.AppImage',
        signature: 'SIGLNX',
      },
    ],
  });
  assert.deepEqual(manifest, {
    version: '0.13.0',
    notes: 'Mejoras de exportación.',
    pub_date: '2026-06-05T12:00:00Z',
    platforms: {
      'windows-x86_64': {
        signature: 'SIGWIN',
        url: 'https://bins.draffity.com/app/0.13.0/X_x64-setup.exe',
      },
      'linux-x86_64': {
        signature: 'SIGLNX',
        url: 'https://bins.draffity.com/app/0.13.0/X_amd64.AppImage',
      },
    },
  });
});

test('buildManifest throws when version is missing', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '',
        pubDate: 'x',
        platforms: [{ key: 'k', url: 'u', signature: 's' }],
      }),
    /version is required/,
  );
});

test('buildManifest throws when there are no platforms', () => {
  assert.throws(
    () => buildManifest({ version: '0.13.0', pubDate: 'x', platforms: [] }),
    /at least one platform/,
  );
});

test('buildManifest throws when a fragment is missing a field', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '0.13.0',
        pubDate: 'x',
        platforms: [{ key: 'windows-x86_64', url: 'u' }],
      }),
    /missing "signature"/,
  );
});

test('buildManifest defaults missing notes to empty string', () => {
  const m = buildManifest({
    version: '0.13.0',
    pubDate: 'x',
    platforms: [{ key: 'k', url: 'u', signature: 's' }],
  });
  assert.equal(m.notes, '');
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `pnpm test:scripts`
Expected: FAIL — `Cannot find module './build-update-manifest.mjs'` (file does not exist yet).

- [ ] **Step 4: Write the minimal implementation**

Create `scripts/build-update-manifest.mjs`:

```js
#!/usr/bin/env node
/**
 * build-update-manifest.mjs
 *
 * Two responsibilities, selected by subcommand:
 *   fragment  — read a single installer's .sig and emit a per-platform fragment
 *               { key, url, signature } as JSON.
 *   assemble  — read all fragments in a directory and emit the Tauri updater
 *               manifest (latest.json) for stable.
 *
 * Used by .github/workflows/release.yml. The pure `buildManifest` is unit-tested.
 * Requires Node >= 20.
 */
import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Pure: build the Tauri updater manifest object.
 * @param {{version: string, notes?: string, pubDate: string, platforms: {key: string, url: string, signature: string}[]}} input
 */
export function buildManifest({ version, notes, pubDate, platforms }) {
  if (!version) throw new Error('version is required');
  if (!Array.isArray(platforms) || platforms.length === 0) {
    throw new Error('at least one platform fragment is required');
  }
  const out = { version, notes: notes ?? '', pub_date: pubDate, platforms: {} };
  for (const p of platforms) {
    if (!p.key) throw new Error('fragment missing "key"');
    if (!p.url) throw new Error(`fragment ${p.key} missing "url"`);
    if (!p.signature) throw new Error(`fragment ${p.key} missing "signature"`);
    out.platforms[p.key] = { signature: p.signature, url: p.url };
  }
  return out;
}

/** Read every *.json fragment in `dir` into an array of objects. */
export function loadFragments(dir) {
  return readdirSync(dir)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(dir, f), 'utf8')));
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    args[argv[i].replace(/^--/, '')] = argv[i + 1];
  }
  return args;
}

function cmdFragment(args) {
  const signature = readFileSync(args['sig-file'], 'utf8').trim();
  const fragment = { key: args.key, url: args.url, signature };
  writeFileSync(args.out, JSON.stringify(fragment, null, 2) + '\n');
}

function cmdAssemble(args) {
  const manifest = buildManifest({
    version: args.version,
    notes: args.notes,
    pubDate: args['pub-date'],
    platforms: loadFragments(args.fragments),
  });
  writeFileSync(args.out, JSON.stringify(manifest, null, 2) + '\n');
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  const args = parseArgs(rest);
  if (cmd === 'fragment') cmdFragment(args);
  else if (cmd === 'assemble') cmdAssemble(args);
  else {
    process.stderr.write(`Unknown command: ${cmd ?? '(none)'}\n`);
    process.exit(1);
  }
}

// Run the CLI only when invoked directly, not when imported by tests.
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `pnpm test:scripts`
Expected: PASS — all 5 `buildManifest` tests green.

- [ ] **Step 6: Commit**

```bash
git add scripts/build-update-manifest.mjs scripts/build-update-manifest.test.mjs package.json
git commit -m "feat(updater): add update-manifest builder with unit tests"
```

---

## Task 3: Manifest builder — `fragment` + `assemble` CLI (TDD)

**Files:**

- Modify: `scripts/build-update-manifest.mjs` (already has the CLI from Task 2)
- Test: `scripts/build-update-manifest.test.mjs`

The implementation already exists (Task 2 Step 4). This task adds an integration test that exercises the `fragment` → `assemble` round-trip through the filesystem, proving the CLI wiring the workflow depends on.

- [ ] **Step 1: Write the failing integration test**

Append to `scripts/build-update-manifest.test.mjs`:

```js
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { loadFragments } from './build-update-manifest.mjs';

test('loadFragments reads every *.json in a directory', () => {
  const dir = mkdtempSync(join(tmpdir(), 'frags-'));
  try {
    writeFileSync(
      join(dir, 'windows-x86_64.json'),
      JSON.stringify({ key: 'windows-x86_64', url: 'u1', signature: 's1' }),
    );
    writeFileSync(
      join(dir, 'linux-x86_64.json'),
      JSON.stringify({ key: 'linux-x86_64', url: 'u2', signature: 's2' }),
    );
    writeFileSync(join(dir, 'ignore.txt'), 'not json');
    const frags = loadFragments(dir).sort((a, b) => a.key.localeCompare(b.key));
    assert.equal(frags.length, 2);
    assert.deepEqual(frags[0], { key: 'linux-x86_64', url: 'u2', signature: 's2' });
    assert.deepEqual(frags[1], { key: 'windows-x86_64', url: 'u1', signature: 's1' });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('fragment subcommand reads a .sig file and writes a fragment json', () => {
  const dir = mkdtempSync(join(tmpdir(), 'frag-cli-'));
  try {
    const sigPath = join(dir, 'app.exe.sig');
    const outPath = join(dir, 'windows-x86_64.json');
    writeFileSync(sigPath, '  SIGCONTENT\n');
    const res = spawnSync(
      process.execPath,
      [
        'scripts/build-update-manifest.mjs',
        'fragment',
        '--key',
        'windows-x86_64',
        '--url',
        'https://bins.draffity.com/app/0.13.0/app.exe',
        '--sig-file',
        sigPath,
        '--out',
        outPath,
      ],
      { encoding: 'utf8' },
    );
    assert.equal(res.status, 0, res.stderr);
    assert.deepEqual(JSON.parse(readFileSync(outPath, 'utf8')), {
      key: 'windows-x86_64',
      url: 'https://bins.draffity.com/app/0.13.0/app.exe',
      signature: 'SIGCONTENT',
    });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
```

Add this import at the top of the test file (next to the other imports):

```js
import { spawnSync } from 'node:child_process';
```

- [ ] **Step 2: Run the test to verify it passes**

Run: `pnpm test:scripts`
Expected: PASS — `loadFragments` and `fragment subcommand` tests green alongside Task 2's.

> Note: the implementation was written in Task 2 Step 4, so these tests should pass immediately. If `fragment subcommand` fails because the script isn't run from repo root, run `pnpm test:scripts` from the repo root (the test invokes `scripts/build-update-manifest.mjs` with a relative path).

- [ ] **Step 3: Commit**

```bash
git add scripts/build-update-manifest.test.mjs
git commit -m "test(updater): cover manifest fragment/assemble CLI round-trip"
```

---

## Task 4: Tauri config + Rust plugins + JS plugin deps

**Files:**

- Modify: `apps/desktop/Cargo.toml`
- Modify: `apps/desktop/src/lib.rs:32-35`
- Modify: `apps/desktop/tauri.conf.json`
- Modify: `apps/desktop/capabilities/default.json`
- Modify: `apps/ui/package.json`

- [ ] **Step 1: Add the Rust crates**

In `apps/desktop/Cargo.toml`, under `[dependencies]` (after the `tauri-plugin-shell = "2"` line):

```toml
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

- [ ] **Step 2: Register the plugins**

In `apps/desktop/src/lib.rs`, replace the plugin block at lines 32-35:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
```

with:

```rust
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
```

- [ ] **Step 3: Configure the updater + NSIS in `tauri.conf.json`**

In `apps/desktop/tauri.conf.json`, replace the `"bundle"` object (lines 30-43) with the following. **Paste the public key from Task 1 Step 1** in place of `PASTE_PUBLIC_KEY_FROM_TASK_1`:

```json
  "bundle": {
    "active": true,
    "targets": ["nsis", "appimage"],
    "createUpdaterArtifacts": true,
    "category": "Productivity",
    "shortDescription": "Asistente de escritura multi-formato",
    "longDescription": "Aplicación de escritura desktop multi-formato (novela, paper, manga, artículo, guion).",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.png",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      }
    }
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://bins.draffity.com/app/stable/latest.json"],
      "pubkey": "PASTE_PUBLIC_KEY_FROM_TASK_1",
      "windows": {
        "installMode": "passive"
      }
    }
  }
```

(The `"plugins"` object is a new sibling of `"bundle"`; add a comma after the closing brace of `"bundle"`.)

- [ ] **Step 4: Grant the capabilities**

In `apps/desktop/capabilities/default.json`, replace the `"permissions"` array:

```json
  "permissions": ["core:default", "dialog:default", "fs:default", "shell:allow-open"]
```

with:

```json
  "permissions": [
    "core:default",
    "dialog:default",
    "fs:default",
    "shell:allow-open",
    "updater:default",
    "process:allow-restart"
  ]
```

- [ ] **Step 5: Add the JS plugin packages**

Run:

```bash
pnpm --filter @draffity/ui add @tauri-apps/plugin-updater @tauri-apps/plugin-process
```

Expected: both appear under `"dependencies"` in `apps/ui/package.json` and `pnpm-lock.yaml` updates.

- [ ] **Step 6: Verify the Rust side compiles (validates capabilities codegen)**

Run: `cargo check --manifest-path apps/desktop/Cargo.toml`
Expected: PASS. `tauri-build` runs during `cargo check` and validates the new permission identifiers; a typo in `updater:default` / `process:allow-restart` would fail here.

- [ ] **Step 7: Commit**

```bash
git add apps/desktop/Cargo.toml apps/desktop/Cargo.lock apps/desktop/src/lib.rs apps/desktop/tauri.conf.json apps/desktop/capabilities/default.json apps/ui/package.json pnpm-lock.yaml
git commit -m "feat(updater): wire tauri-plugin-updater + process, NSIS per-user, R2 endpoint"
```

---

## Task 5: `useUpdater` composable (TDD)

**Files:**

- Create: `apps/ui/src/composables/useUpdater.ts`
- Test: `apps/ui/src/composables/useUpdater.test.ts`

- [ ] **Step 1: Write the failing test**

Create `apps/ui/src/composables/useUpdater.test.ts`:

```ts
import { describe, expect, it, vi, beforeEach } from 'vitest';

const checkMock = vi.fn();
const relaunchMock = vi.fn();

vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch: relaunchMock }));

async function freshUpdater() {
  vi.resetModules();
  const mod = await import('./useUpdater');
  return mod.useUpdater();
}

beforeEach(() => {
  checkMock.mockReset();
  relaunchMock.mockReset();
});

describe('useUpdater', () => {
  it('goes to "available" with version + notes when an update exists', async () => {
    checkMock.mockResolvedValue({
      version: '0.13.0',
      body: 'Novedades',
      downloadAndInstall: vi.fn(),
    });
    const u = await freshUpdater();
    await u.check({ silent: true });
    expect(u.status.value).toBe('available');
    expect(u.availableVersion.value).toBe('0.13.0');
    expect(u.notes.value).toBe('Novedades');
  });

  it('goes to "uptodate" when there is no update', async () => {
    checkMock.mockResolvedValue(null);
    const u = await freshUpdater();
    await u.check({ silent: false });
    expect(u.status.value).toBe('uptodate');
    expect(u.availableVersion.value).toBeNull();
  });

  it('goes to "error" and records the message when check throws', async () => {
    checkMock.mockRejectedValue(new Error('offline'));
    const u = await freshUpdater();
    await u.check({ silent: true });
    expect(u.status.value).toBe('error');
    expect(u.errorMessage.value).toBe('offline');
  });

  it('downloads with progress then becomes "ready", and relaunch calls the process plugin', async () => {
    const downloadAndInstall = vi.fn(async (cb: (e: unknown) => void) => {
      cb({ event: 'Started', data: { contentLength: 200 } });
      cb({ event: 'Progress', data: { chunkLength: 100 } });
      cb({ event: 'Progress', data: { chunkLength: 100 } });
      cb({ event: 'Finished' });
    });
    checkMock.mockResolvedValue({ version: '0.13.0', body: null, downloadAndInstall });
    const u = await freshUpdater();
    await u.check({ silent: true });
    await u.downloadAndInstall();
    expect(u.status.value).toBe('ready');
    expect(u.progress.value).toBe(100);
    await u.relaunchApp();
    expect(relaunchMock).toHaveBeenCalledOnce();
  });

  it('dismiss() hides the banner for the session', async () => {
    checkMock.mockResolvedValue({ version: '0.13.0', body: null, downloadAndInstall: vi.fn() });
    const u = await freshUpdater();
    await u.check({ silent: true });
    expect(u.bannerVisible.value).toBe(true);
    u.dismiss();
    expect(u.bannerVisible.value).toBe(false);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm --filter @draffity/ui test -- useUpdater`
Expected: FAIL — `Cannot find module './useUpdater'`.

- [ ] **Step 3: Write the implementation**

Create `apps/ui/src/composables/useUpdater.ts`:

```ts
/** Auto-update state machine (Tauri 2). Singleton module state so the startup
 *  banner and the Settings panel share one source of truth. Wraps
 *  `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process`. */
import { computed, ref } from 'vue';
import { check as checkUpdate, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdaterStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'uptodate'
  | 'error';

const status = ref<UpdaterStatus>('idle');
const availableVersion = ref<string | null>(null);
const notes = ref<string | null>(null);
const progress = ref(0); // 0..100
const errorMessage = ref<string | null>(null);
const dismissed = ref(false); // dismissed for this session only

let pending: Update | null = null;

const bannerVisible = computed(() => status.value === 'available' && !dismissed.value);

export function useUpdater() {
  /** Check the endpoint. `silent` swallows errors (startup / offline). */
  async function check({ silent }: { silent: boolean }) {
    if (status.value === 'checking' || status.value === 'downloading') return;
    status.value = 'checking';
    errorMessage.value = null;
    try {
      const update = await checkUpdate();
      if (update) {
        pending = update;
        availableVersion.value = update.version;
        notes.value = update.body ?? null;
        dismissed.value = false;
        status.value = 'available';
      } else {
        pending = null;
        availableVersion.value = null;
        notes.value = null;
        status.value = 'uptodate';
      }
    } catch (e) {
      status.value = 'error';
      errorMessage.value = e instanceof Error ? e.message : String(e);
      // `silent` is intentional: a failed startup check must not nag the user.
      void silent;
    }
  }

  /** Download + install the pending update, tracking progress 0..100. */
  async function downloadAndInstall() {
    if (!pending) return;
    status.value = 'downloading';
    progress.value = 0;
    let total = 0;
    let received = 0;
    await pending.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          total = event.data.contentLength ?? 0;
          break;
        case 'Progress':
          received += event.data.chunkLength ?? 0;
          progress.value = total > 0 ? Math.min(100, Math.round((received / total) * 100)) : 0;
          break;
        case 'Finished':
          progress.value = 100;
          break;
      }
    });
    status.value = 'ready';
  }

  /** Restart the app to run the freshly installed version. */
  async function relaunchApp() {
    await relaunch();
  }

  /** Hide the banner until the next launch. */
  function dismiss() {
    dismissed.value = true;
  }

  return {
    status,
    availableVersion,
    notes,
    progress,
    errorMessage,
    bannerVisible,
    check,
    downloadAndInstall,
    relaunchApp,
    dismiss,
  };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter @draffity/ui test -- useUpdater`
Expected: PASS — all 5 cases green.

- [ ] **Step 5: Commit**

```bash
git add apps/ui/src/composables/useUpdater.ts apps/ui/src/composables/useUpdater.test.ts
git commit -m "feat(updater): add useUpdater composable with state-machine tests"
```

---

## Task 6: `UpdateBanner` + startup check wiring (TDD)

**Files:**

- Create: `apps/ui/src/components/UpdateBanner.vue`
- Test: `apps/ui/src/components/UpdateBanner.test.ts`
- Modify: `apps/ui/src/locales/es.json`, `apps/ui/src/locales/en.json`
- Modify: `apps/ui/src/App.vue`

- [ ] **Step 1: Add i18n strings**

In `apps/ui/src/locales/es.json`, add a top-level `"updater"` key (sibling of the other top-level groups):

```json
  "updater": {
    "available": "Draffity {version} disponible",
    "updateNow": "Actualizar ahora",
    "later": "Más tarde",
    "downloading": "Descargando… {percent}%",
    "restart": "Reiniciar para aplicar",
    "checkButton": "Buscar actualizaciones",
    "checking": "Buscando…",
    "upToDate": "Estás al día",
    "error": "No se pudo buscar actualizaciones",
    "currentVersion": "Versión actual",
    "section": "Actualizaciones"
  }
```

In `apps/ui/src/locales/en.json`, add the parallel block:

```json
  "updater": {
    "available": "Draffity {version} available",
    "updateNow": "Update now",
    "later": "Later",
    "downloading": "Downloading… {percent}%",
    "restart": "Restart to apply",
    "checkButton": "Check for updates",
    "checking": "Checking…",
    "upToDate": "You're up to date",
    "error": "Couldn't check for updates",
    "currentVersion": "Current version",
    "section": "Updates"
  }
```

- [ ] **Step 2: Write the failing component test**

Create `apps/ui/src/components/UpdateBanner.test.ts`:

```ts
import { describe, expect, it, vi } from 'vitest';
import { ref, computed } from 'vue';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';

const state = {
  status: ref('available'),
  availableVersion: ref('0.13.0'),
  notes: ref<string | null>('Novedades'),
  progress: ref(0),
  errorMessage: ref<string | null>(null),
  bannerVisible: computed(() => state.status.value === 'available'),
  check: vi.fn(),
  downloadAndInstall: vi.fn(),
  relaunchApp: vi.fn(),
  dismiss: vi.fn(),
};
vi.mock('@/composables/useUpdater', () => ({ useUpdater: () => state }));

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      updater: {
        available: 'Draffity {version} disponible',
        updateNow: 'Actualizar ahora',
        later: 'Más tarde',
        downloading: 'Descargando… {percent}%',
        restart: 'Reiniciar para aplicar',
      },
    },
  },
});

async function mountBanner() {
  const UpdateBanner = (await import('./UpdateBanner.vue')).default;
  return mount(UpdateBanner, { global: { plugins: [i18n] } });
}

describe('UpdateBanner', () => {
  it('shows the available version and triggers download on "Actualizar ahora"', async () => {
    state.status.value = 'available';
    state.progress.value = 0;
    const w = await mountBanner();
    expect(w.text()).toContain('Draffity 0.13.0 disponible');
    await w.get('[data-test="update-now"]').trigger('click');
    expect(state.downloadAndInstall).toHaveBeenCalledOnce();
  });

  it('calls dismiss on "Más tarde"', async () => {
    state.status.value = 'available';
    const w = await mountBanner();
    await w.get('[data-test="update-later"]').trigger('click');
    expect(state.dismiss).toHaveBeenCalledOnce();
  });

  it('shows a restart button once ready', async () => {
    state.status.value = 'ready';
    const w = await mountBanner();
    await w.get('[data-test="update-restart"]').trigger('click');
    expect(state.relaunchApp).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `pnpm --filter @draffity/ui test -- UpdateBanner`
Expected: FAIL — `Cannot find module './UpdateBanner.vue'`.

- [ ] **Step 4: Write the component**

Create `apps/ui/src/components/UpdateBanner.vue` (plain buttons + Tailwind, matching the `AppRail`/Settings nav style so no PrimeVue plugin is needed in tests):

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useUpdater } from '@/composables/useUpdater';

const { t } = useI18n();
const u = useUpdater();
</script>

<template>
  <div
    v-if="u.bannerVisible.value || u.status.value === 'downloading' || u.status.value === 'ready'"
    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 rounded-xl px-4 py-3 shadow-lg bg-surface-0 dark:bg-surface-800 border border-surface-200 dark:border-surface-700 text-sm"
    role="status"
  >
    <!-- AVAILABLE -->
    <template v-if="u.status.value === 'available'">
      <span class="font-medium">{{
        t('updater.available', { version: u.availableVersion.value })
      }}</span>
      <button
        data-test="update-now"
        type="button"
        class="px-3 py-1 rounded-lg bg-primary-500 text-white hover:bg-primary-600 transition-colors"
        @click="u.downloadAndInstall()"
      >
        {{ t('updater.updateNow') }}
      </button>
      <button
        data-test="update-later"
        type="button"
        class="px-3 py-1 rounded-lg text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-700 transition-colors"
        @click="u.dismiss()"
      >
        {{ t('updater.later') }}
      </button>
    </template>

    <!-- DOWNLOADING -->
    <template v-else-if="u.status.value === 'downloading'">
      <span>{{ t('updater.downloading', { percent: u.progress.value }) }}</span>
      <div class="w-32 h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
        <div
          class="h-full bg-primary-500 transition-all"
          :style="{ width: u.progress.value + '%' }"
        />
      </div>
    </template>

    <!-- READY -->
    <template v-else-if="u.status.value === 'ready'">
      <button
        data-test="update-restart"
        type="button"
        class="px-3 py-1 rounded-lg bg-primary-500 text-white hover:bg-primary-600 transition-colors"
        @click="u.relaunchApp()"
      >
        {{ t('updater.restart') }}
      </button>
    </template>
  </div>
</template>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `pnpm --filter @draffity/ui test -- UpdateBanner`
Expected: PASS — all 3 cases green.

- [ ] **Step 6: Mount the banner + fire the startup check in `App.vue`**

In `apps/ui/src/App.vue`, add the import after the other component imports (after line 9, `import CommandPalette ...`):

```ts
import UpdateBanner from '@/components/UpdateBanner.vue';
import { useUpdater } from '@/composables/useUpdater';
```

Add the composable instance after `const ui = useUiStore();` (line 18):

```ts
const updater = useUpdater();
```

Inside the existing `onMounted(() => { ... })` (starts line 25), add as the first statement (a silent check; failures are swallowed by the composable):

```ts
void updater.check({ silent: true });
```

In the `<template>`, add `<UpdateBanner />` as a sibling of `<CommandPalette />` (just before the closing `</div>` of `.draffity-app`):

```vue
<CommandPalette />
<UpdateBanner />
```

- [ ] **Step 7: Verify typecheck + tests pass**

Run: `pnpm --filter @draffity/ui typecheck && pnpm --filter @draffity/ui test -- UpdateBanner useUpdater`
Expected: PASS — no type errors, banner + composable tests green.

- [ ] **Step 8: Commit**

```bash
git add apps/ui/src/components/UpdateBanner.vue apps/ui/src/components/UpdateBanner.test.ts apps/ui/src/locales/es.json apps/ui/src/locales/en.json apps/ui/src/App.vue
git commit -m "feat(updater): add update banner + silent startup check"
```

---

## Task 7: `SettingsUpdates` panel in the "about" section

**Files:**

- Create: `apps/ui/src/components/SettingsUpdates.vue`
- Modify: `apps/ui/src/views/Settings.vue`

- [ ] **Step 1: Write the component**

Create `apps/ui/src/components/SettingsUpdates.vue`:

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import { getVersion } from '@tauri-apps/api/app';
import { useUpdater } from '@/composables/useUpdater';

const { t } = useI18n();
const u = useUpdater();
const currentVersion = ref('');

onMounted(async () => {
  try {
    currentVersion.value = await getVersion();
  } catch {
    // best-effort: outside Tauri (e.g. browser dev) getVersion is unavailable.
  }
});
</script>

<template>
  <section class="flex items-center justify-between gap-4">
    <div>
      <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
        {{ t('updater.section') }}
      </h2>
      <p class="text-xs opacity-60 mt-1">
        {{ t('updater.currentVersion') }}: <span class="font-mono">{{ currentVersion }}</span>
      </p>
      <p v-if="u.status.value === 'uptodate'" class="text-xs opacity-60 mt-1">
        {{ t('updater.upToDate') }}
      </p>
      <p v-else-if="u.status.value === 'available'" class="text-xs opacity-80 mt-1">
        {{ t('updater.available', { version: u.availableVersion.value }) }}
      </p>
      <p v-else-if="u.status.value === 'error'" class="text-xs text-red-500 mt-1">
        {{ t('updater.error') }}
      </p>
    </div>
    <Button
      :label="u.status.value === 'checking' ? t('updater.checking') : t('updater.checkButton')"
      icon="pi pi-sync"
      size="small"
      severity="secondary"
      :loading="u.status.value === 'checking'"
      @click="u.check({ silent: false })"
    />
  </section>
</template>
```

- [ ] **Step 2: Mount it in the Settings "about" section**

In `apps/ui/src/views/Settings.vue`, add the import after `import SettingsStats ...` (line 20):

```ts
import SettingsUpdates from '@/components/SettingsUpdates.vue';
```

In the template, inside the `about` section (the `<div v-show="activeSection === 'about'" ...>` block, line 489), add `<SettingsUpdates />` as the first child, before the crash-reporting `<section>`:

```vue
        <div v-show="activeSection === 'about'" class="space-y-8">
          <SettingsUpdates />
          <section v-if="crashReportingActive" class="flex items-center justify-between gap-4">
```

- [ ] **Step 3: Verify typecheck passes**

Run: `pnpm --filter @draffity/ui typecheck`
Expected: PASS — no type errors.

- [ ] **Step 4: Commit**

```bash
git add apps/ui/src/components/SettingsUpdates.vue apps/ui/src/views/Settings.vue
git commit -m "feat(updater): add manual check + current version in Settings"
```

---

## Task 8: Release pipeline — sign, upload to R2, assemble manifest

**Files:**

- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Add the signing env to the `tauri-action` step**

In `.github/workflows/release.yml`, the `env:` of the "Build & publish with tauri-action" step currently has only `GITHUB_TOKEN`. Replace that `env:` block with:

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

- [ ] **Step 2: Upload the installer + emit a manifest fragment (per matrix job)**

Add these two steps at the end of the `publish` job's `steps:` (after the `tauri-action` step):

```yaml
- name: Upload installer to R2 + build manifest fragment
  shell: bash
  env:
    AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
    AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
    AWS_DEFAULT_REGION: auto
    R2_ENDPOINT: ${{ secrets.R2_ENDPOINT }}
  run: |
    set -euo pipefail
    VERSION="${GITHUB_REF_NAME#v}"
    BUNDLE="apps/desktop/target/release/bundle"
    if [ "$RUNNER_OS" = "Windows" ]; then
      KEY="windows-x86_64"
      INSTALLER=$(ls "$BUNDLE"/nsis/*-setup.exe | head -n1)
    else
      KEY="linux-x86_64"
      INSTALLER=$(ls "$BUNDLE"/appimage/*.AppImage | head -n1)
    fi
    BASENAME=$(basename "$INSTALLER")
    echo "Uploading $BASENAME -> s3://bins-draffity/app/$VERSION/"
    aws s3 cp "$INSTALLER" "s3://bins-draffity/app/$VERSION/$BASENAME" --endpoint-url "$R2_ENDPOINT"
    mkdir -p fragments
    node scripts/build-update-manifest.mjs fragment \
      --key "$KEY" \
      --url "https://bins.draffity.com/app/$VERSION/$BASENAME" \
      --sig-file "$INSTALLER.sig" \
      --out "fragments/$KEY.json"

- name: Upload manifest fragment artifact
  uses: actions/upload-artifact@v4
  with:
    name: update-fragment-${{ runner.os }}
    path: fragments/*.json
    if-no-files-found: error
```

- [ ] **Step 3: Add the manifest aggregation job**

At the end of `.github/workflows/release.yml`, add a second job (sibling of `publish`):

```yaml
manifest:
  name: Assemble & publish update manifest
  needs: publish
  runs-on: ubuntu-latest
  steps:
    - name: Checkout
      uses: actions/checkout@v4

    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: '20'

    - name: Download manifest fragments
      uses: actions/download-artifact@v4
      with:
        path: fragments
        pattern: update-fragment-*
        merge-multiple: true

    - name: Build latest.json
      shell: bash
      run: |
        set -euo pipefail
        VERSION="${GITHUB_REF_NAME#v}"
        PUB_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        node scripts/build-update-manifest.mjs assemble \
          --fragments fragments \
          --version "$VERSION" \
          --notes "Draffity $VERSION" \
          --pub-date "$PUB_DATE" \
          --out latest.json
        echo "----- latest.json -----"
        cat latest.json

    - name: Publish latest.json to R2
      shell: bash
      env:
        AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
        AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
        AWS_DEFAULT_REGION: auto
        R2_ENDPOINT: ${{ secrets.R2_ENDPOINT }}
      run: |
        aws s3 cp latest.json s3://bins-draffity/app/stable/latest.json --endpoint-url "$R2_ENDPOINT"
```

- [ ] **Step 4: Verify the workflow is valid YAML**

Run: `node -e "const fs=require('fs');const yaml=fs.readFileSync('.github/workflows/release.yml','utf8');require('js-yaml').load(yaml);console.log('valid yaml')"`
Expected: prints `valid yaml`. (`js-yaml` is already in the dependency tree — used by ESLint.) If it's not resolvable, instead open the file and confirm the two jobs (`publish`, `manifest`) are sibling keys under `jobs:` with consistent indentation.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci(release): sign installers, vendor to R2, publish update manifest"
```

---

## Task 9: Flip the AUTO-UPDATE doc status + verify the whole suite

**Files:**

- Modify: `docs/AUTO-UPDATE.md`

- [ ] **Step 1: Update the status banner in `docs/AUTO-UPDATE.md`**

Replace the status blockquote at the top (lines 3-6) with:

```markdown
> **Estado: IMPLEMENTADO.** Auto-update activo para Windows (NSIS per-user) y
> Linux (AppImage), sirviendo manifiesto + instaladores firmados desde
> `bins.draffity.com/app/`. Diseño: `docs/specs/2026-06-05-auto-update-tauri-design.md`.
> Pendiente: macOS y firma OS-level (diferidos). Repo: `aresoftlabs/draffity`.
```

- [ ] **Step 2: Run the full test + lint suite**

Run: `pnpm test:scripts && pnpm --filter @draffity/ui test && pnpm --filter @draffity/ui typecheck && pnpm lint:js`
Expected: all PASS.

- [ ] **Step 3: Commit**

```bash
git add docs/AUTO-UPDATE.md
git commit -m "docs(updater): mark auto-update as implemented"
```

---

## Task 10: End-to-end staging verification (manual)

This validates the real update cycle without touching real users. It runs **after** Tasks 1-9 are merged. Requires the version in config to match the tag.

- [ ] **Step 1: Build + install a baseline version locally**

Confirm `apps/desktop/tauri.conf.json` `version` is the current `0.12.0`. Build and install:

```bash
pnpm tauri:build
```

Install the produced NSIS installer from `apps/desktop/target/release/bundle/nsis/`. Launch it.

- [ ] **Step 2: Stage a higher version's artifacts under `app/staging/`**

Temporarily point the app at staging: in `apps/desktop/tauri.conf.json`, set `plugins.updater.endpoints` to `["https://bins.draffity.com/app/staging/latest.json"]` and bump `version` to `0.13.0` (also `Cargo.toml`, root `package.json`, `apps/ui/package.json`). Build, then manually upload the `0.13.0` installer + `.sig` to `s3://bins-draffity/app/0.13.0/` and assemble a `latest.json` to `app/staging/latest.json` using the same script:

```bash
node scripts/build-update-manifest.mjs fragment --key windows-x86_64 \
  --url https://bins.draffity.com/app/0.13.0/<installer>.exe \
  --sig-file apps/desktop/target/release/bundle/nsis/<installer>.exe.sig \
  --out fragments/windows-x86_64.json
node scripts/build-update-manifest.mjs assemble --fragments fragments \
  --version 0.13.0 --notes "staging test" --pub-date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --out latest.json
aws s3 cp latest.json s3://bins-draffity/app/staging/latest.json --endpoint-url "$R2_ENDPOINT"
```

- [ ] **Step 3: Confirm the update cycle on the installed baseline**

Launch the installed `0.12.0` build (the one whose endpoint points to staging). Expected: the banner appears ("Draffity 0.13.0 disponible"). Click "Actualizar ahora" → progress bar → "Reiniciar para aplicar" → app relaunches as `0.13.0` with **no UAC prompt**. Verify the new version in Settings → Acerca de → Actualizaciones.

- [ ] **Step 4: Revert the staging endpoint**

Restore `plugins.updater.endpoints` to `["https://bins.draffity.com/app/stable/latest.json"]`. Do **not** commit the staging endpoint change. The real release flow (bump version → tag `vX.Y.Z` → `release.yml`) publishes to `app/stable/` automatically.

> **Release reminder:** to ship version `X.Y.Z`, the built artifact's version (from `tauri.conf.json`) must equal the tag. Bump `version` in `apps/desktop/tauri.conf.json`, `apps/desktop/Cargo.toml`, root `package.json`, and `apps/ui/package.json`, commit, then `git tag vX.Y.Z && git push --tags`.

---

## Self-Review

**Spec coverage:**

- Plataformas Windows + Linux → Task 4 (targets `nsis`, `appimage`), Task 8 (per-OS upload). ✓
- Endpoint estático en R2 → Task 4 (`endpoints`), Task 8 (upload + manifest). ✓
- NSIS per-user → Task 4 (`bundle.windows.nsis.installMode: currentUser`, `plugins.updater.windows.installMode: passive`). ✓
- Firma minisign → Task 1 (keygen + secrets), Task 4 (`createUpdaterArtifacts`, `pubkey`), Task 8 (signing env). ✓
- Layout R2 `app/<v>/` + `app/stable/latest.json` → Task 8. ✓
- `latest.json` shape → Task 2 (`buildManifest`). ✓
- Job de agregación → Task 8 Step 3. ✓
- GitHub Release sin cambios → Task 8 leaves the `tauri-action` release config untouched. ✓
- UX: chequeo al inicio + banner no intrusivo → Task 6; botón manual + versión actual → Task 7. ✓
- Testing: script del manifiesto → Tasks 2-3; `useUpdater` state machine → Task 5; E2E staging → Task 10. ✓
- Capabilities `updater:default` + `process:allow-restart` → Task 4 Step 4. ✓

**Placeholder scan:** `PASTE_PUBLIC_KEY_FROM_TASK_1` is the only token — it is a value produced by Task 1 Step 1 and placed in Task 4 Step 3, not a deferred TODO. No other placeholders.

**Type consistency:** `UpdaterStatus` values (`idle|checking|available|downloading|ready|uptodate|error`) used identically in `useUpdater.ts`, its test, `UpdateBanner.vue`, and `SettingsUpdates.vue`. The composable's exposed names (`status`, `availableVersion`, `notes`, `progress`, `errorMessage`, `bannerVisible`, `check`, `downloadAndInstall`, `relaunchApp`, `dismiss`) match every consumer and the test mock. The script's `buildManifest`/`loadFragments`/`fragment`/`assemble` names are consistent across Tasks 2, 3, and 8.
