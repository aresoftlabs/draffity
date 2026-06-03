import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useLabelStore } from './labels';
import type { Label } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function mkLabel(over: Partial<Label> = {}): Label {
  return {
    id: over.id ?? 'l1',
    projectId: over.projectId ?? 'p1',
    name: over.name ?? 'Importante',
    color: over.color ?? '#ef4444',
    createdAt: over.createdAt ?? 0,
  };
}

describe('useLabelStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('loadFor populates labels and byId map', async () => {
    const labels = [mkLabel({ id: 'a', name: 'A' }), mkLabel({ id: 'b', name: 'B' })];
    invokeMock.mockResolvedValueOnce(labels);
    const store = useLabelStore();
    await store.loadFor('p1');

    expect(store.labels.length).toBe(2);
    expect(store.byId.get('b')?.name).toBe('B');
  });

  it('create appends and keeps the list sorted by name', async () => {
    invokeMock.mockResolvedValueOnce([mkLabel({ id: 'z', name: 'Zeta' })]);
    const store = useLabelStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(mkLabel({ id: 'a', name: 'alfa' }));
    await store.create({ projectId: 'p1', name: 'alfa', color: '#000000' });

    expect(store.labels.map((l) => l.name)).toEqual(['alfa', 'Zeta']);
  });

  it('remove drops the label from the cache', async () => {
    invokeMock.mockResolvedValueOnce([mkLabel({ id: 'a' }), mkLabel({ id: 'b' })]);
    const store = useLabelStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(undefined);
    await store.remove('a');

    expect(store.labels.map((l) => l.id)).toEqual(['b']);
    expect(store.byId.has('a')).toBe(false);
  });

  it('update replaces the label and re-sorts', async () => {
    invokeMock.mockResolvedValueOnce([mkLabel({ id: 'a', name: 'alfa' })]);
    const store = useLabelStore();
    await store.loadFor('p1');

    invokeMock.mockResolvedValueOnce(mkLabel({ id: 'a', name: 'omega', color: '#00ff00' }));
    await store.update('a', 'omega', '#00ff00');

    expect(store.byId.get('a')?.name).toBe('omega');
    expect(store.byId.get('a')?.color).toBe('#00ff00');
  });

  it('reset clears state', async () => {
    invokeMock.mockResolvedValueOnce([mkLabel()]);
    const store = useLabelStore();
    await store.loadFor('p1');
    store.reset();
    expect(store.labels).toEqual([]);
  });
});
