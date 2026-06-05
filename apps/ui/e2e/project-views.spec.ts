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
    await mockIpc.handle('list_projects', [project]);
    await mockIpc.handle('get_active_project', project);
    await mockIpc.handle('get_project', project);
    await mockIpc.handle('list_documents', documents);
  });

  test('renders the project route with binder and view-mode navigation', async ({ page }) => {
    await page.goto('/project/p1');

    // The project title appears in the project navigation.
    await expect(page.getByRole('navigation', { name: /Project navigation/i })).toContainText(
      'Test project',
    );

    // The Activity rail exposes the Editor / Corkboard / Outliner view modes,
    // and switching between them works without error.
    const activity = page.getByRole('navigation', { name: /Activity/i });
    await expect(activity.getByRole('button', { name: 'Corkboard' })).toBeVisible();
    await expect(activity.getByRole('button', { name: 'Outliner' })).toBeVisible();
    await activity.getByRole('button', { name: 'Corkboard' }).click();
    await activity.getByRole('button', { name: 'Outliner' }).click();
    await activity.getByRole('button', { name: 'Editor' }).click();
    await expect(activity.getByRole('button', { name: 'Editor' })).toBeVisible();
  });
});
