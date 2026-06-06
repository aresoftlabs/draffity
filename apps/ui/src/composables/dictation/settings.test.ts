import { afterEach, describe, expect, it } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { resolveAsrModelId, resolveInputDeviceId, resolveAutoStop } from './settings';
import { resolveDictationMode } from './settings';
import { resolveVoiceLanguage } from './settings';
import { setLocale } from '@/locales';

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

describe('resolveDictationMode', () => {
  it('defaults to manual and reads streaming from the store', () => {
    setActivePinia(createPinia());
    expect(resolveDictationMode()).toBe('manual');
    useVoiceSettingsStore().dictationMode = 'streaming';
    expect(resolveDictationMode()).toBe('streaming');
  });
  it('falls back to manual without active pinia', () => {
    setActivePinia(undefined as never);
    expect(resolveDictationMode()).toBe('manual');
  });
});

describe('resolveVoiceLanguage', () => {
  it('follows the app locale when no override', () => {
    setActivePinia(createPinia());
    setLocale('fr');
    expect(resolveVoiceLanguage()).toBe('fr');
  });
  it('uses the voice override when set (locale or auto)', () => {
    setActivePinia(createPinia());
    setLocale('fr');
    useVoiceSettingsStore().voiceLanguage = 'es';
    expect(resolveVoiceLanguage()).toBe('es');
    useVoiceSettingsStore().voiceLanguage = 'auto';
    expect(resolveVoiceLanguage()).toBe('auto');
  });
});
