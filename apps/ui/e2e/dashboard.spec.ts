import { test, expect, dismissOnboarding } from './fixtures';

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await dismissOnboarding(page);
  });

  test('renders empty state when there are no projects', async ({ page }) => {
    await page.goto('/');
    // The empty-state CTA appears.
    await expect(page.getByRole('button', { name: /New Draft/i }).first()).toBeVisible();
  });

  test('opens the New Project wizard and shows step 1 (template picker)', async ({
    page,
    mockIpc,
  }) => {
    await mockIpc.handle('list_templates', () => [
      {
        schemaVersion: 1,
        id: 'generic',
        name: 'Mock Generic',
        description: 'For tests',
        kind: 'generic',
        locale: 'es',
        structure: [{ title: 'Capítulo 1', docType: 'chapter' }],
        metadataFields: [],
      },
    ]);

    await page.goto('/');
    await page
      .getByRole('button', { name: /New Draft/i })
      .first()
      .click();

    // Wizard step 1 (template picker) shows the selectable template card.
    await expect(page.getByText('Mock Generic')).toBeVisible();
    await expect(page.getByRole('tab', { name: /Template/i })).toBeVisible();
  });
});
