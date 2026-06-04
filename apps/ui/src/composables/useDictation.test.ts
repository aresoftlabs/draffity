import { afterEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { ref } from 'vue';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

import { resolveAsrModelId, useDictation } from './useDictation';

// Capturamos el handler que useDictation registra para el evento de progreso.
let progressHandler: ((e: { payload: { progress: number } }) => void) | null = null;
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((name: string, cb: (e: { payload: { progress: number } }) => void) => {
    if (name === 'voice.transcribe.progress') progressHandler = cb;
    return Promise.resolve(() => {});
  }),
}));

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

describe('useDictation progress', () => {
  afterEach(() => {
    progressHandler = null;
  });

  it('exposes a progress ref updated by the transcribe-progress event', async () => {
    setActivePinia(createPinia());
    const editor = ref(null);
    const dictation = useDictation(editor);
    // Esperar a que la suscripción async se registre.
    await Promise.resolve();
    await Promise.resolve();
    expect(dictation.progress.value).toBeNull();
    // El handler solo aplica durante la transcripción (ignora eventos rezagados).
    dictation.phase.value = 'transcribing';
    progressHandler?.({ payload: { progress: 37 } });
    expect(dictation.progress.value).toBe(37);
    // Fuera de 'transcribing' los eventos no mueven el valor.
    dictation.phase.value = 'idle';
    progressHandler?.({ payload: { progress: 80 } });
    expect(dictation.progress.value).toBe(37);
  });
});
