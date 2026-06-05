import { describe, expect, it, vi, beforeEach } from 'vitest';

const checkMock = vi.fn();
const relaunchMock = vi.fn();

vi.mock('@tauri-apps/plugin-updater', () => ({ check: checkMock }));
vi.mock('@tauri-apps/plugin-process', () => ({ relaunch: relaunchMock }));

// `vi.resetModules()` gives each test a fresh useUpdater module (new singleton
// state). The `vi.mock()` factories above are hoisted and file-scoped, so they
// keep intercepting the re-imported module — `checkMock`/`relaunchMock` stay
// wired across resets. Don't call `vi.unmock`/`vi.restoreAllMocks` here or the
// re-import would hit the real Tauri plugins.
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

  it('surfaces "error" on a manual check when check throws', async () => {
    checkMock.mockRejectedValue(new Error('offline'));
    const u = await freshUpdater();
    await u.check({ silent: false });
    expect(u.status.value).toBe('error');
    expect(u.errorMessage.value).toBe('offline');
  });

  it('stays idle (no nag) on a silent check failure but records the message', async () => {
    checkMock.mockRejectedValue(new Error('offline'));
    const u = await freshUpdater();
    await u.check({ silent: true });
    expect(u.status.value).toBe('idle');
    expect(u.errorMessage.value).toBe('offline');
  });

  it('ignores a second check() while one is already in flight', async () => {
    let resolveCheck!: (v: null) => void;
    checkMock.mockImplementation(() => new Promise((res) => (resolveCheck = res)));
    const u = await freshUpdater();
    const first = u.check({ silent: true });
    expect(u.status.value).toBe('checking');
    await u.check({ silent: true }); // second call must no-op
    expect(u.status.value).toBe('checking');
    expect(checkMock).toHaveBeenCalledOnce();
    resolveCheck(null);
    await first;
    expect(u.status.value).toBe('uptodate');
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

  it('keeps progress at 0 while downloading when contentLength is unknown', async () => {
    const downloadAndInstall = vi.fn(async (cb: (e: unknown) => void) => {
      cb({ event: 'Started', data: {} });
      cb({ event: 'Progress', data: { chunkLength: 50 } });
    });
    checkMock.mockResolvedValue({ version: '0.13.0', body: null, downloadAndInstall });
    const u = await freshUpdater();
    await u.check({ silent: true });
    await u.downloadAndInstall();
    expect(u.progress.value).toBe(0);
    expect(u.status.value).toBe('ready');
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
