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
