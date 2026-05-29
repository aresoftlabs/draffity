import AxeBuilder from '@axe-core/playwright';
import { test, expect, dismissOnboarding } from './fixtures';

/**
 * D-07 — axe-core sweeps. Each spec navigates to a route, lets the UI
 * settle, and asserts there are no WCAG 2.0/2.1 AA violations. We only
 * scope to the `tags` Playwright recognises by default and the wider
 * `best-practice` set is intentionally excluded — it surfaces issues
 * that aren't conformance failures (e.g. heading-order on intentional
 * design choices) and would block CI without buying the user anything.
 *
 * The scans target the rendered HTML the dev server serves, which the
 * Tauri runtime mocks via the IPC fixture. That means we're checking
 * UI markup quality (labels, ARIA roles, color contrast) — not how the
 * native shell renders the same page. The two are equivalent for a11y
 * purposes since the WebView uses the same DOM.
 */

const AA_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

async function runAxe(page: import('@playwright/test').Page) {
  return new AxeBuilder({ page }).withTags(AA_TAGS).analyze();
}

test.describe('a11y', () => {
  test.beforeEach(async ({ page }) => {
    await dismissOnboarding(page);
  });

  test('Dashboard route has no WCAG AA violations', async ({ page }) => {
    await page.goto('/');
    // Wait for the empty-state CTA to be visible — the route is stable
    // once that button paints.
    await expect(
      page.getByRole('button', { name: /Crear nuevo proyecto|Create new project/i }),
    ).toBeVisible();

    const results = await runAxe(page);
    expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
  });

  test('Settings route has no WCAG AA violations', async ({ page, mockIpc }) => {
    // Default mocks don't include the daily-writing chart — wire a
    // minimal handler so the panel renders an empty series instead of
    // throwing.
    await mockIpc.handle('get_recent_daily_writing', () => []);
    await mockIpc.handle('list_backups', () => []);
    await mockIpc.handle('get_crash_reporting_status', () => ({
      active: false,
      enabled: false,
    }));

    await page.goto('/settings');
    await expect(page.getByRole('heading', { name: /Settings|Ajustes/i })).toBeVisible();

    const results = await runAxe(page);
    expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
  });
});
