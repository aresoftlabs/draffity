import { afterEach, describe, expect, it } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { resolveAsrModelId, resolveInputDeviceId, resolveAutoStop } from './settings';

describe('dictation settings resolvers', () => {
  afterEach(() => localStorage.clear());

  it('reads asrModelId / inputDeviceId / autoStopOnSilence from the store', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.asrModelId = 'base.en';
    store.inputDeviceId = 'mic-1';
    store.autoStopOnSilence = true;
    expect(resolveAsrModelId()).toBe('base.en');
    expect(resolveInputDeviceId()).toBe('mic-1');
    expect(resolveAutoStop()).toBe(true);
  });

  it('falls back safely when there is no active pinia', () => {
    // Sin pinia activa: useVoiceSettingsStore() lanza y los resolvers caen al default.
    setActivePinia(undefined);
    expect(resolveAsrModelId()).toBeNull();
    expect(resolveInputDeviceId()).toBeNull();
    expect(resolveAutoStop()).toBe(false);
  });
});
