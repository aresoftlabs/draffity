import { test, expect } from './fixtures';

test.describe('Onboarding', () => {
  // Each test runs in a fresh browser context, so localStorage starts empty
  // (the onboarding flag is absent) — no need to clear it. Crucially we must
  // NOT clear it via addInitScript, which would re-run on reload and wipe the
  // flag the wizard just persisted.

  test('shows the welcome dialog on first launch', async ({ page }) => {
    await page.goto('/');
    // Welcome heading is rendered inside the modal.
    await expect(page.getByRole('heading', { name: /Draffity|Welcome/i })).toBeVisible();
  });

  test('completing the wizard persists the flag and never re-appears', async ({ page }) => {
    await page.goto('/');

    // Advance through the 3 slides via the primary button: "Next", "Next",
    // then "Create my first project" (which finishes and persists the flag).
    const advance = page.getByRole('button', { name: /^Next$|Create my first project/i });
    await advance.click();
    await advance.click();
    await advance.click();

    // The onboarding welcome dialog should be gone now.
    await expect(page.getByRole('heading', { name: /Welcome to Draffity/i })).toHaveCount(0);

    // Reload — flag is persisted, so the dialog stays away.
    await page.reload();
    await expect(page.getByRole('heading', { name: /Welcome to Draffity/i })).toHaveCount(0);
  });
});
