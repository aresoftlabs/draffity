import { afterEach, describe, expect, it } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

import { resolveAsrModelId } from './useDictation';

describe('resolveAsrModelId', () => {
  afterEach(() => {
    localStorage.clear();
  });

  it('returns asrModelId from the store when set', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.asrModelId = 'base.en';
    expect(resolveAsrModelId()).toBe('base.en');
  });

  it('returns null when asrModelId is null', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.asrModelId = null;
    expect(resolveAsrModelId()).toBeNull();
  });
});
