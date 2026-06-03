import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import { useMediaStore } from './media';

const invokeMock = vi.mocked(invoke);

describe('useMediaStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('mime returns undefined for unknown id', () => {
    const store = useMediaStore();
    expect(store.mime('missing')).toBeUndefined();
  });

  it('forget is a no-op for unknown id', () => {
    const store = useMediaStore();
    expect(() => store.forget('missing')).not.toThrow();
  });

  it('reset clears urls cache', () => {
    const store = useMediaStore();
    store.urls = new Map([['x', 'blob:http://localhost/x']]);
    store.reset();
    expect(store.urls.size).toBe(0);
  });
});
