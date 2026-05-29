import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useCustomFieldStore } from './customFields';
import type { CustomField } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function mkField(over: Partial<CustomField> = {}): CustomField {
  return {
    id: over.id ?? 'f1',
    projectId: over.projectId ?? 'p1',
    name: over.name ?? 'POV',
    kind: over.kind ?? 'text',
    options: over.options ?? [],
    position: over.position ?? 0,
    createdAt: over.createdAt ?? 0,
  };
}

describe('useCustomFieldStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('loadFor populates fields in backend order', async () => {
    const fields = [mkField({ id: 'a', position: 0 }), mkField({ id: 'b', position: 1 })];
    invokeMock.mockResolvedValueOnce(fields);
    const store = useCustomFieldStore();
    await store.loadFor('p1');
    expect(store.fields.map((f) => f.id)).toEqual(['a', 'b']);
  });

  it('create appends the new field', async () => {
    invokeMock.mockResolvedValueOnce([]);
    const store = useCustomFieldStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(mkField({ id: 'new', name: 'Due', kind: 'date' }));
    await store.create({ projectId: 'p1', name: 'Due', kind: 'date' });
    expect(store.fields).toHaveLength(1);
    expect(store.fields[0].name).toBe('Due');
  });

  it('update replaces the field in place', async () => {
    invokeMock.mockResolvedValueOnce([mkField({ id: 'a', name: 'Old' })]);
    const store = useCustomFieldStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(mkField({ id: 'a', name: 'New' }));
    await store.update('a', 'New', []);
    expect(store.fields[0].name).toBe('New');
  });

  it('remove drops the field', async () => {
    invokeMock.mockResolvedValueOnce([mkField({ id: 'a' }), mkField({ id: 'b' })]);
    const store = useCustomFieldStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(undefined);
    await store.remove('a');
    expect(store.fields.map((f) => f.id)).toEqual(['b']);
  });

  it('reset clears state', async () => {
    invokeMock.mockResolvedValueOnce([mkField()]);
    const store = useCustomFieldStore();
    await store.loadFor('p1');
    store.reset();
    expect(store.fields).toEqual([]);
  });
});
