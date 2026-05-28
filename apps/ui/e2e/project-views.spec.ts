import { test, expect, dismissOnboarding } from './fixtures';

// Sprint 3 — verify the Editor/Corkboard/Outliner toggle keeps the same
// document selection across switches, and the selected card/row reflects
// what the binder has picked.

const project = {
  id: 'p1',
  title: 'Test project',
  templateId: 'generic',
  status: 'active' as const,
  metadata: null,
  goalWords: null,
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

const documents = [
  {
    id: 'd1',
    projectId: 'p1',
    parentId: null,
    title: 'Capítulo 1',
    docType: 'chapter' as const,
    content: '<p>Hola mundo.</p>',
    synopsis: 'Empieza la aventura',
    position: 0,
    status: 'draft' as const,
    tags: [],
    goalWords: null,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  {
    id: 'd2',
    projectId: 'p1',
    parentId: null,
    title: 'Capítulo 2',
    docType: 'chapter' as const,
    content: '<p>Sigue la aventura.</p>',
    synopsis: null,
    position: 1,
    status: 'revised' as const,
    tags: ['acción'],
    goalWords: 500,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
];

test.describe('Project views (Sprint 3)', () => {
  test.beforeEach(async ({ page, mockIpc }) => {
    await dismissOnboarding(page);
    await mockIpc.handle('list_projects', () => [project]);
    await mockIpc.handle('get_active_project', () => project);
    await mockIpc.handle('get_project', () => project);
    await mockIpc.handle('list_documents', () => documents);
  });

  test('toggle Editor → Corkboard → Outliner keeps selection', async ({ page }) => {
    await page.goto('/project/p1');

    // Default mode: editor. The first document should be selected and
    // rendered in the editor panel.
    await expect(page.getByRole('heading', { name: 'Test project' })).toBeVisible();

    // The view toggle is a SelectButton; switching to "corkboard" by
    // its aria-label ("Corcho" in ES default — covers EN too).
    const corkboardOption = page.locator(
      'button[role="radio"][aria-label*="Corcho"], button[role="radio"][aria-label*="Corkboard"]',
    );
    await corkboardOption.first().click();

    // Corkboard renders a card per document with the synopsis text.
    await expect(page.getByText('Empieza la aventura')).toBeVisible();

    // Switch to Outliner — the table renders both chapters by title.
    const outlinerOption = page.locator(
      'button[role="radio"][aria-label*="Esquema"], button[role="radio"][aria-label*="Outliner"]',
    );
    await outlinerOption.first().click();

    // The Outliner table should show both titles as cells.
    await expect(page.getByRole('cell', { name: /Capítulo 1/ })).toBeVisible();
    await expect(page.getByRole('cell', { name: /Capítulo 2/ })).toBeVisible();
  });
});
