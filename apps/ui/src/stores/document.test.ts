import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { countWords, useDocumentStore } from './document';
import type { DocNode } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function makeDoc(over: Partial<DocNode> = {}): DocNode {
  const now = Date.now();
  return {
    id: over.id ?? 'd1',
    projectId: over.projectId ?? 'p1',
    parentId: over.parentId ?? null,
    title: over.title ?? 'Doc',
    docType: over.docType ?? 'chapter',
    content: over.content ?? null,
    synopsis: over.synopsis ?? null,
    position: over.position ?? 0,
    status: over.status ?? 'draft',
    tags: over.tags ?? [],
    labelIds: over.labelIds ?? [],
    metadata: over.metadata ?? {},
    isResearch: over.isResearch ?? false,
    isFrontMatter: over.isFrontMatter ?? false,
    isBackMatter: over.isBackMatter ?? false,
    createdAt: over.createdAt ?? now,
    updatedAt: over.updatedAt ?? now,
  };
}

describe('countWords', () => {
  it('returns 0 for empty input', () => {
    expect(countWords('')).toBe(0);
  });

  it('strips HTML tags and counts words', () => {
    expect(countWords('<p>Hola mundo</p>')).toBe(2);
    expect(countWords('<h1>Capítulo 1</h1><p>Una frase con cinco palabras.</p>')).toBe(7);
  });

  it('treats nbsp and whitespace correctly', () => {
    expect(countWords('a&nbsp;b&nbsp;c')).toBe(3);
  });
});

describe('useDocumentStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('loadFor selects the first document', async () => {
    const docs = [makeDoc({ id: 'a', position: 0 }), makeDoc({ id: 'b', position: 1 })];
    invokeMock.mockResolvedValueOnce(docs);

    const store = useDocumentStore();
    await store.loadFor('p1');

    expect(store.documents.length).toBe(2);
    expect(store.selectedId).toBe('a');
  });

  it('save flips state idle → saving → saved and updates the doc', async () => {
    const initial = makeDoc({ id: 'x', content: 'old' });
    invokeMock.mockResolvedValueOnce([initial]);
    const store = useDocumentStore();
    await store.loadFor('p1');

    expect(store.saveState).toBe('idle');

    invokeMock.mockResolvedValueOnce({ ...initial, content: 'new' });
    await store.save('x', { content: 'new' });

    expect(store.saveState).toBe('saved');
    expect(store.lastSavedAt).not.toBeNull();
    expect(store.documents[0].content).toBe('new');
  });

  it('save sets state to error when invoke rejects', async () => {
    invokeMock.mockResolvedValueOnce([makeDoc()]);
    const store = useDocumentStore();
    await store.loadFor('p1');

    invokeMock.mockRejectedValueOnce(new Error('boom'));
    await expect(store.save('d1', { content: 'x' })).rejects.toThrow('boom');
    expect(store.saveState).toBe('error');
  });

  it('totalWordCount sums across documents stripping HTML', async () => {
    const docs = [
      makeDoc({ id: 'a', content: '<p>uno dos</p>' }),
      makeDoc({ id: 'b', content: '<h2>tres</h2>' }),
    ];
    invokeMock.mockResolvedValueOnce(docs);
    const store = useDocumentStore();
    await store.loadFor('p1');
    expect(store.totalWordCount).toBe(3);
  });

  it('totalWordCount excludes research docs and their descendants', async () => {
    const docs = [
      makeDoc({ id: 'ch', content: '<p>uno dos</p>' }),
      makeDoc({ id: 'res', isResearch: true, content: '<p>tres</p>' }),
      makeDoc({ id: 'note', parentId: 'res', content: '<p>cuatro cinco seis</p>' }),
    ];
    invokeMock.mockResolvedValueOnce(docs);
    const store = useDocumentStore();
    await store.loadFor('p1');
    // Only "uno dos" counts; the research folder + its nested note are excluded.
    expect(store.totalWordCount).toBe(2);
    expect(store.researchIds.has('note')).toBe(true);
    expect(store.researchIds.has('ch')).toBe(false);
  });

  it('reset clears state', async () => {
    invokeMock.mockResolvedValueOnce([makeDoc()]);
    const store = useDocumentStore();
    await store.loadFor('p1');
    store.reset();
    expect(store.documents).toEqual([]);
    expect(store.selectedId).toBeNull();
  });
});
