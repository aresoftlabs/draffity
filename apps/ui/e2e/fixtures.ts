import { test as base, type Page } from '@playwright/test';

/**
 * IPC mock plumbing. Each Playwright test starts the Vue app in a Vite dev
 * server, but the real Tauri runtime never exists in the browser — so every
 * `invoke()` would fail. We patch `window.__TAURI_INTERNALS__` *before* the
 * bundle loads (`addInitScript`) with a programmable handler map.
 *
 * The mock is installed on EVERY test via a `page` fixture override, so tests
 * that only need the defaults don't have to opt in. Tests that need custom
 * responses register them via `mockIpc.handle(name, fn)` — which also uses
 * `addInitScript`, so overrides survive navigation and may be registered
 * before the first `goto()`.
 */
export interface IpcMock {
  /**
   * Register a mock response for an IPC command. `response` may be either:
   *  - a JSON-serializable value (recommended for static data), or
   *  - a SELF-CONTAINED function (no closures — it is serialized via
   *    `fn.toString()` and rebuilt in the browser, so it cannot reference
   *    variables from the test's scope).
   */
  handle: (name: string, response: unknown) => Promise<void>;
}

/** Installed before the app bundle runs, on every navigation. */
function installIpcMock() {
  const w = window as unknown as {
    __TAURI_INTERNALS__: unknown;
    __mockIpcHandlers: Record<string, (args?: unknown) => unknown>;
  };
  if (w.__mockIpcHandlers) return; // already installed in this document

  const handlers: Record<string, (args?: unknown) => unknown> = {
    ping: () => 'pong (mock)',
    list_projects: () => [],
    get_active_project: () => null,
    get_writing_stats: () => ({ currentStreak: 0, longestStreak: 0, lastWritingDate: null }),
    get_recent_daily_writing: () => [],
    get_setting: () => null,
    set_setting: () => null,
    list_templates: () => [],
  };
  w.__mockIpcHandlers = handlers;
  w.__TAURI_INTERNALS__ = {
    invoke: (cmd: string, args?: unknown) => {
      const h = w.__mockIpcHandlers[cmd];
      if (!h) {
        return Promise.reject({ code: 'unsupported', message: `mock: no handler for ${cmd}` });
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
}

export const test = base.extend<{ mockIpc: IpcMock }>({
  // Auto-install the IPC mock on every test, before any page script runs.
  page: async ({ page }, use) => {
    await page.addInitScript(installIpcMock);
    await use(page);
  },

  mockIpc: async ({ page }, use) => {
    const mockIpc: IpcMock = {
      handle: async (name, response) => {
        // addInitScript (not page.evaluate) so the override survives navigation
        // and can be registered before the first goto(). Runs after the default
        // install script (registered earlier by the page fixture).
        if (typeof response === 'function') {
          await page.addInitScript(
            ({ name, fnSrc }) => {
              const w = window as unknown as {
                __mockIpcHandlers: Record<string, (args?: unknown) => unknown>;
              };
              w.__mockIpcHandlers = w.__mockIpcHandlers || {};
              const fn = new Function('args', `return (${fnSrc}).call(null, args)`);
              w.__mockIpcHandlers[name] = (args) => fn(args);
            },
            { name, fnSrc: (response as (a?: unknown) => unknown).toString() },
          );
        } else {
          // Static JSON value — serialized so it survives the page boundary
          // without depending on closures.
          await page.addInitScript(
            ({ name, json }) => {
              const w = window as unknown as {
                __mockIpcHandlers: Record<string, (args?: unknown) => unknown>;
              };
              w.__mockIpcHandlers = w.__mockIpcHandlers || {};
              w.__mockIpcHandlers[name] = () => JSON.parse(json);
            },
            { name, json: JSON.stringify(response ?? null) },
          );
        }
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
