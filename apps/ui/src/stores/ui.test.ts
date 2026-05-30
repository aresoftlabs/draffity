import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

// Stub theme side-effects so the store can load without a real DOM.
vi.mock('@/styles/theme', () => ({
  getStoredTheme: () => 'system',
  setTheme: vi.fn(),
}));
vi.mock('@/locales', () => ({ setLocale: vi.fn() }));

import { useUiStore } from './ui';

describe('useUiStore – toggleLightDark', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('switches light → dark', () => {
    const store = useUiStore();
    store.setTheme('light');
    store.toggleLightDark();
    expect(store.theme).toBe('dark');
  });

  it('switches dark → light', () => {
    const store = useUiStore();
    store.setTheme('dark');
    store.toggleLightDark();
    expect(store.theme).toBe('light');
  });

  it('resolves high-contrast as dark and lands on light', () => {
    const store = useUiStore();
    store.setTheme('high-contrast');
    store.toggleLightDark();
    expect(store.theme).toBe('light');
  });
});
