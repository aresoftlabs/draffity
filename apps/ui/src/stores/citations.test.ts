import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useCitationsStore, surnameOf, labelFor } from './citations';
import type { Citation } from '@draffity/shared-types';

const invokeMock = vi.mocked(invoke);

function mkCitation(over: Partial<Citation> = {}): Citation {
  return {
    key: over.key ?? 'test2024',
    fields: over.fields ?? { author: 'Doe, John', title: 'A Test', year: '2024' },
  };
}

describe('surnameOf', () => {
  it('extracts surname from "Last, First" format', () => {
    expect(surnameOf(mkCitation())).toBe('Doe');
  });

  it('extracts last token from "First Last" format', () => {
    const c = mkCitation({ fields: { author: 'John Smith', title: 'X', year: '2024' } });
    expect(surnameOf(c)).toBe('Smith');
  });

  it('handles "and"-joined authors (first only)', () => {
    const c = mkCitation({
      fields: { author: 'Doe, John and Smith, Jane', title: 'X', year: '2024' },
    });
    expect(surnameOf(c)).toBe('Doe');
  });

  it('returns empty string for missing author', () => {
    const c = mkCitation({ fields: { author: '', title: 'X', year: '2024' } });
    expect(surnameOf(c)).toBe('');
  });
});

describe('labelFor', () => {
  it('formats (Surname, Year) when both present', () => {
    expect(labelFor(mkCitation())).toBe('(Doe, 2024)');
  });

  it('falls back to (Surname) when year is missing', () => {
    expect(labelFor(mkCitation({ fields: { author: 'Doe, John', title: 'X', year: '' } }))).toBe(
      '(Doe)',
    );
  });

  it('falls back to (Year) when surname is missing', () => {
    expect(labelFor(mkCitation({ fields: { author: '', title: 'X', year: '2024' } }))).toBe(
      '(2024)',
    );
  });

  it('falls back to [key] when both are missing', () => {
    expect(labelFor(mkCitation({ fields: { author: '', title: 'X', year: '' } }))).toBe(
      '[test2024]',
    );
  });
});

describe('useCitationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('reset clears projectId and list', () => {
    const store = useCitationsStore();
    store.projectId = 'p1';
    store.setList([mkCitation()]);
    store.reset();
    expect(store.projectId).toBeNull();
    expect(store.list).toEqual([]);
  });

  it('loadFor fetches from ipc on first call', async () => {
    invokeMock.mockResolvedValueOnce([mkCitation()]);
    const store = useCitationsStore();
    await store.loadFor('p1');
    expect(store.list).toHaveLength(1);
    expect(store.list[0].key).toBe('test2024');
  });

  it('loadFor skips fetch when same project already loaded', async () => {
    invokeMock.mockResolvedValueOnce([mkCitation()]);
    const store = useCitationsStore();
    await store.loadFor('p1');
    invokeMock.mockClear();
    await store.loadFor('p1');
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it('byKey builds a lookup map', async () => {
    invokeMock.mockResolvedValueOnce([mkCitation({ key: 'a' }), mkCitation({ key: 'b' })]);
    const store = useCitationsStore();
    await store.loadFor('p1');
    expect(store.byKey.get('a')).toBeDefined();
    expect(store.byKey.get('b')).toBeDefined();
    expect(store.byKey.has('c')).toBe(false);
  });
});
