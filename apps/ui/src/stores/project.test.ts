import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useProjectStore } from './project';
import type { Project } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function makeProject(over: Partial<Project> = {}): Project {
  const now = Date.now();
  return {
    id: over.id ?? 'p1',
    title: over.title ?? 'Proyecto',
    templateId: over.templateId ?? 'generic',
    status: over.status ?? 'active',
    metadata: null,
    createdAt: over.createdAt ?? now,
    updatedAt: over.updatedAt ?? now,
  };
}

describe('useProjectStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('loadAll populates projects and active id', async () => {
    const a = makeProject({ id: 'a', status: 'active' });
    const b = makeProject({ id: 'b', status: 'archived' });
    // list_projects â†’ [a, b], get_active_project â†’ a
    invokeMock.mockResolvedValueOnce([a, b]);
    invokeMock.mockResolvedValueOnce(a);

    const store = useProjectStore();
    await store.loadAll();

    expect(store.projects.length).toBe(2);
    expect(store.active?.id).toBe('a');
    expect(store.archived).toEqual([b]);
    expect(store.activeId).toBe('a');
  });

  it('archived computed filters by status', async () => {
    const a = makeProject({ id: 'a', status: 'archived' });
    const b = makeProject({ id: 'b', status: 'archived' });
    invokeMock.mockResolvedValueOnce([a, b]);
    invokeMock.mockResolvedValueOnce(null);

    const store = useProjectStore();
    await store.loadAll();
    expect(store.archived.length).toBe(2);
    expect(store.active).toBeNull();
  });

  it('isCurrentReadOnly reflects status of selectedLocally', async () => {
    const a = makeProject({ id: 'a', status: 'active' });
    const b = makeProject({ id: 'b', status: 'archived' });
    invokeMock.mockResolvedValueOnce([a, b]);
    invokeMock.mockResolvedValueOnce(a);

    const store = useProjectStore();
    await store.loadAll();

    store.selectLocally('b');
    expect(store.current?.id).toBe('b');
    expect(store.isCurrentReadOnly).toBe(true);

    store.selectLocally('a');
    expect(store.isCurrentReadOnly).toBe(false);
  });

  it('create returns the project and reloads', async () => {
    const newProject = makeProject({ id: 'new', title: 'X' });
    // create_project â†’ newProject
    invokeMock.mockResolvedValueOnce(newProject);
    // loadAll() â†’ list + active
    invokeMock.mockResolvedValueOnce([newProject]);
    invokeMock.mockResolvedValueOnce(newProject);

    const store = useProjectStore();
    const result = await store.create({ title: 'X', templateId: 'generic' });
    expect(result.id).toBe('new');
    expect(store.activeId).toBe('new');
  });
});
