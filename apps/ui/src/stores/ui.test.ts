import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

// Stub theme side-effects so the store can load without a real DOM.
vi.mock('@/styles/theme', () => ({
  getStoredTheme: () => 'system',
  setTheme: vi.fn(),
}));
vi.mock('@/locales', () => ({ setLocale: vi.fn() }));

import { useUiStore } from './ui';

describe('useUiStore – cycleTheme', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('advances through system → light → dark cycle', () => {
    const store = useUiStore();
    // Initial theme from mock is 'system'.
    expect(store.theme).toBe('system');

    store.cycleTheme();
    expect(store.theme).toBe('light');

    store.cycleTheme();
    expect(store.theme).toBe('dark');

    store.cycleTheme();
    expect(store.theme).toBe('system');
  });

  it('wraps around from dark back to system', () => {
    const store = useUiStore();
    store.cycleTheme(); // system → light
    store.cycleTheme(); // light  → dark
    store.cycleTheme(); // dark   → system
    expect(store.theme).toBe('system');
  });
});
