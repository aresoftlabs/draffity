import { test, expect } from './fixtures';

test.describe('Onboarding', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any persisted flag from previous runs.
    await page.addInitScript(() => {
      window.localStorage.removeItem('draffity.onboarded');
    });
  });

  test('shows the welcome dialog on first launch', async ({ page }) => {
    await page.goto('/');
    // Welcome heading is rendered inside the modal.
    await expect(page.getByRole('heading', { name: /Draffity|Welcome/i })).toBeVisible();
  });

  test('completing the wizard persists the flag and never re-appears', async ({ page }) => {
    await page.goto('/');

    // Click "Next/Siguiente" twice and "Start/Empezar" on the last step.
    const nextOrFinish = page.getByRole('button', {
      name: /Siguiente|Empezar|Next|Start/,
    });
    await nextOrFinish.click();
    await nextOrFinish.click();
    await nextOrFinish.click();

    // The dialog should be gone now.
    await expect(page.getByRole('heading', { name: /Bienvenido|Welcome/i })).toHaveCount(0);

    // Reload — flag is persisted, so the dialog stays away.
    await page.reload();
    await expect(page.getByRole('heading', { name: /Bienvenido|Welcome/i })).toHaveCount(0);
  });
});
