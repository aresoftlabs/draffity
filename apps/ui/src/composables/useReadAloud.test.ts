import { afterEach, describe, expect, it } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

// Import the pure function we'll create
import { resolveVoiceId } from './useReadAloud';

describe('resolveVoiceId', () => {
  afterEach(() => {
    localStorage.clear();
  });

  it('returns ttsVoiceId from the store when set', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = 'es_ES-carlfm';
    expect(resolveVoiceId()).toBe('es_ES-carlfm');
  });

  it('falls back to empty string when ttsVoiceId is null', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    expect(resolveVoiceId()).toBe('');
  });
});
