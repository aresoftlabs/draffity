import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import { createPinia, setActivePinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import SettingsVoice from './SettingsVoice.vue';

// The component subscribes to a Tauri event on mount; stub it.
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock('@tauri-apps/plugin-fs', () => ({ readFile: vi.fn() }));

let mediaRecorderMock: {
  start: ReturnType<typeof vi.fn>;
  stop: ReturnType<typeof vi.fn>;
  state: string;
  ondataavailable: ((e: { data: Blob }) => void) | null;
  onstop: (() => void) | null;
};

const audioContextMock = {
  createMediaStreamSource: vi.fn(() => ({ connect: vi.fn(), disconnect: vi.fn() })),
  createScriptProcessor: vi.fn(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
    onaudioprocess: null as ((e: unknown) => void) | null,
  })),
  close: vi.fn(),
  sampleRate: 44100,
  destination: {},
};

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
        voiceModelSelector: 'Selector de modelo ASR',
        voiceModelSelectorLabel: 'Modelo de dictado',
        voiceModelNotInstalled: 'No instalado',
        voiceDiskLabel: 'En disco',
        voiceDiskUsage: 'Datos de voz: {count} modelos — {size}',
        voiceDiskNone: 'Sin datos de voz en disco',
        voiceTestPlay: 'Probar voz',
        voiceTestPlaying: 'Reproduciendo…',
        voiceTestInstallFirst: 'Instalar voz primero',
        voiceAsrTestRecord: 'Grabar prueba',
        voiceAsrTestStop: 'Detener',
        voiceAsrTestResult: 'Transcripción',
        voiceAsrTestMicDenied: 'Micrófono requerido',
        voiceAsrTestLabel: 'Probar dictado',
        voiceCatalog: 'Catálogo de voces',
        voiceCatalogByLang: 'Voces en {lang}',
        voiceCatalogEmpty: 'No hay voces disponibles',
        voiceCatalogRetry: 'Reintentar',
        voiceCatalogSize: '{size} MB',
        voiceDownloadFailed: 'Descarga fallida',
      },
    },
  },
});

function mountVoice() {
  return mount(SettingsVoice, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

describe('SettingsVoice', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
    mediaRecorderMock = {
      start: vi.fn(),
      stop: vi.fn(),
      state: 'inactive',
      ondataavailable: null,
      onstop: null,
    };
    vi.stubGlobal(
      'MediaRecorder',
      vi.fn(() => mediaRecorderMock),
    );
    vi.stubGlobal(
      'Blob',
      vi.fn(() => ({})),
    );
    vi.stubGlobal(
      'FileReader',
      vi.fn(() => ({
        readAsArrayBuffer: vi.fn(),
        onloadend: null as (() => void) | null,
        result: new ArrayBuffer(8),
      })),
    );
    vi.stubGlobal(
      'AudioContext',
      vi.fn(() => audioContextMock),
    );
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:test'),
      revokeObjectURL: vi.fn(),
    });
    Object.defineProperty(globalThis.navigator, 'mediaDevices', {
      value: {
        getUserMedia: vi.fn().mockResolvedValue({
          getTracks: vi.fn(() => [{ stop: vi.fn() }]),
        }),
      },
      writable: true,
    });
  });

  it('lists models and binary status when voice is enabled', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models')
        return Promise.resolve([
          { id: 'ggml-base', sizeMb: 142, recommended: true, installed: false },
        ]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Voz');
    expect(w.text()).toContain('Whisper instalado');
    expect(w.text()).toContain('ggml-base');
    expect(w.text()).toContain('Descargar');
  });

  it('shows ASR model selector with installed voice models', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models')
        return Promise.resolve([
          { id: 'ggml-base', sizeMb: 142, recommended: true, installed: true },
          { id: 'ggml-small', sizeMb: 466, recommended: false, installed: true },
        ]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    // Should show the selector heading
    expect(w.text()).toContain('Selector de modelo ASR');
    // Should show the select label referencing the store-wired model
    expect(w.text()).toContain('Modelo de dictado');
    // Should NOT have "not installed" badge when no model selected
    expect(w.text()).not.toContain('No instalado');
  });

  it('shows "not installed" badge when selected ASR model is not in installed list', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models')
        return Promise.resolve([
          { id: 'ggml-base', sizeMb: 142, recommended: true, installed: true },
        ]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    // Pre-set store with a model that is NOT in the installed list
    const { useVoiceSettingsStore } = await import('@/stores/voiceSettings');
    const store = useVoiceSettingsStore();
    store.asrModelId = 'ggml-large';
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('No instalado');
  });

  it('shows disk usage summary line when models are installed', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: true });
      if (cmd === 'list_voice_models')
        return Promise.resolve([
          { id: 'ggml-base', sizeMb: 142, recommended: true, installed: true },
        ]);
      if (cmd === 'list_voice_voices')
        return Promise.resolve([
          {
            id: 'es_ES-carlfm',
            name: 'Carl (FM)',
            lang: 'es',
            sizeMb: 42,
            recommended: true,
            installed: true,
          },
        ]);
      if (cmd === 'get_disk_usage')
        return Promise.resolve([
          { id: 'ggml-base', bytes: 148_897_792 },
          { id: 'es_ES-carlfm', bytes: 44_040_192 },
        ]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    // Summary line shows count and human-readable size
    expect(w.text()).toContain('Datos de voz');
    expect(w.text()).toMatch(/2 modelos/);
    // Should show per-model breakdown
    expect(w.text()).toContain('En disco');
    expect(w.text()).toContain('142');
    expect(w.text()).toContain('42');
  });

  it('shows "no voice data" when disk is empty', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Sin datos de voz en disco');
  });

  it('shows play button per installed voice', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: true });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices')
        return Promise.resolve([
          {
            id: 'es_ES-carlfm',
            name: 'Carl (FM)',
            lang: 'es',
            sizeMb: 42,
            recommended: true,
            installed: true,
          },
          {
            id: 'en_US-amy',
            name: 'Amy (US)',
            lang: 'en',
            sizeMb: 35,
            recommended: false,
            installed: false,
          },
        ]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    // Installed voice should have a pi-play icon (play button for testing)
    const playBtns = w.findAll('button').filter((b) => b.text().includes('Probar voz'));
    expect(playBtns).toHaveLength(1);
  });

  it('calls testSynthesize on play button click', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: true });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices')
        return Promise.resolve([
          {
            id: 'es_ES-carlfm',
            name: 'Carl (FM)',
            lang: 'es',
            sizeMb: 42,
            recommended: true,
            installed: true,
          },
        ]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      if (cmd === 'test_synthesize') return Promise.resolve('/tmp/test_output.wav');
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    const playBtn = w.findAll('button').find((b) => b.text().includes('Probar voz'));
    expect(playBtn).toBeDefined();
    await playBtn!.trigger('click');
    await flushPromises();
    // Verify test_synthesize was called with the right args
    const synthCall = invokeMock.mock.calls.find((c) => c[0] === 'test_synthesize');
    expect(synthCall).toBeDefined();
    expect(synthCall![1]).toEqual({
      voiceId: 'es_ES-carlfm',
      text: 'Hello, this is a test voice.',
    });
  });

  it('shows ASR test recorder section', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Probar dictado');
    expect(w.text()).toContain('Grabar prueba');
  });

  it('shows record/stop button for ASR test', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: false });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    const recordBtn = w.findAll('button').find((b) => b.text().includes('Grabar prueba'));
    expect(recordBtn).toBeDefined();
  });

  it('shows voice catalog section with language groups', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: true });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      if (cmd === 'list_available_models')
        return Promise.resolve([
          {
            lang: 'es',
            items: [
              {
                id: 'es_ES-carlfm',
                name: 'Carl (FM)',
                lang: 'es',
                sizeMb: 42,
                recommended: true,
                installed: false,
                diskBytes: 0,
                kind: 'voice',
              },
            ],
          },
          {
            lang: 'en',
            items: [
              {
                id: 'en_US-amy',
                name: 'Amy (US)',
                lang: 'en',
                sizeMb: 35,
                recommended: false,
                installed: true,
                diskBytes: 36_700_160,
                kind: 'voice',
              },
            ],
          },
        ]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Catálogo de voces');
    expect(w.text()).toContain('Carl (FM)');
    expect(w.text()).toContain('Amy (US)');
    expect(w.text()).toContain('Instalada');
    expect(w.text()).toContain('Descargar');
  });

  it('shows empty catalog message', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_voice_status')
        return Promise.resolve({ binaryInstalled: true, piperInstalled: true });
      if (cmd === 'list_voice_models') return Promise.resolve([]);
      if (cmd === 'list_voice_voices') return Promise.resolve([]);
      if (cmd === 'get_disk_usage') return Promise.resolve([]);
      if (cmd === 'list_available_models') return Promise.resolve([]);
      return Promise.resolve(undefined);
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('No hay voces disponibles');
  });
});
