import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useCodexStore } from './codex';
import type { CodexEntry, CodexKind } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function mkEntry(over: Partial<CodexEntry> = {}): CodexEntry {
  return {
    id: over.id ?? 'e1',
    projectId: over.projectId ?? 'p1',
    kind: over.kind ?? ('character' as CodexKind),
    name: over.name ?? 'Alice',
    body: over.body ?? '<p>Protagonist</p>',
    tags: over.tags ?? ['main'],
    createdAt: over.createdAt ?? 0,
  };
}

describe('useCodexStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  describe('computed maps', () => {
    beforeEach(async () => {
      invokeMock.mockResolvedValueOnce([
        mkEntry({ id: 'e1', name: 'Alice' }),
        mkEntry({ id: 'e2', name: 'Bob', tags: ['side'] }),
        mkEntry({ id: 'e3', name: 'Charlie', tags: ['main', 'villain'] }),
      ]);
      const store = useCodexStore();
      await store.loadFor('p1');
    });

    it('byId builds id lookup', () => {
      const store = useCodexStore();
      expect(store.byId.get('e1')?.name).toBe('Alice');
      expect(store.byId.has('missing')).toBe(false);
    });

    it('byNameLower builds lowercase name index', () => {
      const store = useCodexStore();
      expect(store.byNameLower.get('alice')?.id).toBe('e1');
      expect(store.byNameLower.get('bob')?.id).toBe('e2');
    });

    it('allTags collects unique tags alphabetically', () => {
      const store = useCodexStore();
      expect(store.allTags).toEqual(['main', 'side', 'villain']);
    });
  });

  describe('filtered', () => {
    beforeEach(async () => {
      invokeMock.mockResolvedValueOnce([
        mkEntry({
          id: 'e1',
          name: 'Alice',
          kind: 'character',
          tags: ['main'],
          body: '<p>Main character</p>',
        }),
        mkEntry({
          id: 'e2',
          name: 'Wonderland',
          kind: 'place',
          tags: ['fantasy'],
          body: '<p>Fantasy world</p>',
        }),
        mkEntry({
          id: 'e3',
          name: 'Cheshire Cat',
          kind: 'character',
          tags: ['main', 'animal'],
          body: '<p>Smiling cat</p>',
        }),
      ]);
      const store = useCodexStore();
      await store.loadFor('p1');
    });

    it('returns all when query is empty and no filters', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: '', kind: null, tag: null })).toHaveLength(3);
    });

    it('filters by kind', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: '', kind: 'place', tag: null })).toHaveLength(1);
    });

    it('filters by tag', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: '', kind: null, tag: 'animal' })).toHaveLength(1);
    });

    it('filters by query (name match)', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: 'ali', kind: null, tag: null })).toHaveLength(1);
    });

    it('filters by query (body match)', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: 'fantasy', kind: null, tag: null })).toHaveLength(1);
    });

    it('filters by query (tag match)', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: 'animal', kind: null, tag: null })).toHaveLength(1);
    });

    it('combines kind + query', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: '', kind: 'character', tag: 'main' })).toHaveLength(2);
    });

    it('returns empty when nothing matches', () => {
      const store = useCodexStore();
      expect(store.filtered({ query: 'xyz', kind: null, tag: null })).toHaveLength(0);
    });
  });

  describe('CRUD', () => {
    it('reset clears projectId and entries', async () => {
      invokeMock.mockResolvedValueOnce([mkEntry()]);
      const store = useCodexStore();
      await store.loadFor('p1');
      store.reset();
      expect(store.projectId).toBeNull();
      expect(store.entries).toEqual([]);
    });

    it('remove deletes via IPC and removes from local list', async () => {
      invokeMock.mockResolvedValueOnce([mkEntry({ id: 'a' }), mkEntry({ id: 'b' })]);
      const store = useCodexStore();
      await store.loadFor('p1');
      invokeMock.mockResolvedValueOnce(undefined);
      await store.remove('a');
      expect(store.entries.map((e) => e.id)).toEqual(['b']);
    });
  });
});
