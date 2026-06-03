import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import SettingsVoice from './SettingsVoice.vue';

// The component subscribes to a Tauri event on mount; stub it.
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        voiceTitle: 'Voz',
        voiceHint: 'hint',
        voiceBinaryInstalled: 'Whisper instalado',
        voiceBinaryMissing: 'Falta Whisper',
        voiceImportBinary: 'Importar binario',
        voiceRecommended: 'Recomendado',
        voiceModelDownload: 'Descargar',
        voiceModelDelete: 'Eliminar',
        voiceReadAloud: 'Lectura en voz alta',
        voicePiperInstalled: 'Piper instalado',
        voicePiperMissing: 'Falta Piper',
        voiceImportPiper: 'Importar Piper',
        voiceInstalled: 'Instalada',
      },
      capability: { unavailable: 'No disponible' },
    },
  },
});

function mountVoice() {
  return mount(SettingsVoice, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

describe('SettingsVoice', () => {
  beforeEach(() => invokeMock.mockReset());

  it('lists models and binary status when voice is enabled', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models')
        return Promise.resolve([
          { id: 'ggml-base', sizeMb: 142, recommended: true, installed: false },
        ]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Voz');
    expect(w.text()).toContain('Whisper instalado');
    expect(w.text()).toContain('ggml-base');
    expect(w.text()).toContain('Descargar');
  });
});
