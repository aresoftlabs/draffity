import { test as base, type Page } from '@playwright/test';

/**
 * IPC mock plumbing. Each Playwright test starts the Vue app in a Vite dev
 * server, but the real Tauri runtime never exists in the browser — so every
 * `invoke()` would fail. We patch `window.__TAURI_INTERNALS__` *before* the
 * bundle loads (`addInitScript`) with a programmable handler map.
 *
 * Tests register the commands they care about via `mockIpc.handle(name, fn)`;
 * unhandled commands resolve to a sensible default (empty list / null) so
 * the UI doesn't crash on incidental calls.
 */
export interface IpcMock {
  handle: (name: string, fn: (args?: unknown) => unknown) => Promise<void>;
}

export const test = base.extend<{ mockIpc: IpcMock }>({
  mockIpc: async ({ page }, use) => {
    await page.addInitScript(() => {
      const handlers: Record<string, (args?: unknown) => unknown> = {};

      // Default no-op behaviours for incidental calls.
      handlers['ping'] = () => 'pong (mock)';
      handlers['list_projects'] = () => [];
      handlers['get_active_project'] = () => null;
      handlers['get_writing_stats'] = () => ({
        currentStreak: 0,
        longestStreak: 0,
        lastWritingDate: null,
      });
      handlers['get_setting'] = () => null;
      handlers['set_setting'] = () => null;
      handlers['list_templates'] = () => [];

      // Persisted across the page lifetime so tests can override at runtime.
      const w = window as unknown as {
        __TAURI_INTERNALS__: unknown;
        __mockIpcHandlers: typeof handlers;
      };
      w.__mockIpcHandlers = handlers;
      w.__TAURI_INTERNALS__ = {
        invoke: (cmd: string, args?: unknown) => {
          const h = w.__mockIpcHandlers[cmd];
          if (!h) {
            return Promise.reject({
              code: 'unsupported',
              message: `mock: no handler for ${cmd}`,
            });
          }
          try {
            return Promise.resolve(h(args));
          } catch (e) {
            return Promise.reject(e);
          }
        },
        metadata: {
          currentWindow: { label: 'main' },
          currentWebview: { label: 'main', windowLabel: 'main' },
        },
      };
    });

    const mockIpc: IpcMock = {
      handle: async (name, fn) => {
        await page.evaluate(
          ({ name, fnSrc }) => {
            const w = window as unknown as {
              __mockIpcHandlers: Record<string, (args?: unknown) => unknown>;
            };
            const fn = new Function('args', `return (${fnSrc}).call(null, args)`);
            w.__mockIpcHandlers[name] = (args) => fn(args);
          },
          { name, fnSrc: fn.toString() },
        );
      },
    };

    await use(mockIpc);
  },
});

/** Re-export `expect` so specs only import from this module. */
export { expect } from '@playwright/test';

/**
 * Convenience: clear the onboarding flag before navigation when the test
 * relies on the Dashboard being reachable on first paint.
 */
export async function dismissOnboarding(page: Page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem('draffity.onboarded', '1');
    } catch {
      /* ignore */
    }
  });
}
