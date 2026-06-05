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
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

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
  createBuffer: vi.fn((_ch: number, len: number) => ({
    getChannelData: () => new Float32Array(len),
  })),
  createBufferSource: vi.fn(() => ({
    buffer: null as unknown,
    connect: vi.fn(),
    start: vi.fn(),
    onended: null as (() => void) | null,
  })),
  resume: vi.fn(),
  close: vi.fn(),
  state: 'running',
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
        voiceDictation: 'Dictado',
        voiceManageModels: 'Administrar modelos',
        voiceBinaryInstalled: 'Whisper instalado',
        voiceBinaryMissing: 'Falta Whisper',
        voiceBinaryImported: 'Binario importado',
        voiceDownloadBinary: 'Descargar binario',
        voiceImportBinary: 'Importar binario',
        voiceRecommended: 'Recomendado',
        voiceModelDownload: 'Descargar',
        voiceModelDownloaded: 'Descargado',
        voiceModelDelete: 'Eliminar',
        voiceModelError: 'Error de descarga',
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
        voiceTestError: 'Error al probar',
        voiceAsrTestRecord: 'Grabar prueba',
        voiceAsrTestStop: 'Detener',
        voiceAsrTestResult: 'Transcripción',
        voiceAsrTestMicDenied: 'Micrófono requerido',
        voiceAsrTestLabel: 'Probar dictado',
        voiceInputDevice: 'Micrófono',
        voiceInputDeviceDefault: 'Micrófono predeterminado del sistema',
        voiceInputDeviceEmpty: 'No se detectaron micrófonos',
        voiceInputDeviceUnnamed: 'Micrófono {n}',
        voiceInputDeviceRefresh: 'Actualizar lista de micrófonos',
        voiceInputDeviceHint: 'Elegí qué micrófono usar para el dictado.',
        voiceCatalogSearch: 'Buscar voz',
        voiceCatalogAllLangs: 'Otros idiomas ({count})',
        voiceDelete: 'Eliminar voz',
        voiceAutoStopSilence: 'Detener tras silencio',
        voiceAutoStopSilenceHint: 'Finaliza la grabación sola tras silencio.',
        voiceAccel: 'Aceleración',
        voiceAccelModel: 'Modelo activo',
        voiceAccelRedetect: 'Re-detectar',
        voiceAccelMetal: 'Metal (GPU Apple)',
        voiceAccelVulkan: 'Vulkan (GPU)',
        voiceAccelCpu: 'CPU',
      },
    },
  },
});

function mountVoice() {
  return mount(SettingsVoice, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

function stubInvoke(over: Partial<Record<string, unknown>> = {}) {
  const defaults: Record<string, unknown> = {
    get_voice_status: { binaryInstalled: true, piperInstalled: false },
    list_voice_models: [],
    get_disk_usage: [],
    get_accel_status: null,
    get_voice_catalog: [],
  };
  const table = { ...defaults, ...over };
  invokeMock.mockImplementation((cmd: string) =>
    cmd in table ? Promise.resolve(table[cmd]) : Promise.resolve(undefined),
  );
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
        enumerateDevices: vi.fn().mockResolvedValue([]),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      },
      writable: true,
    });
  });

  it('renders the section title and both child blocks', async () => {
    stubInvoke();
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Voz');
    expect(w.findComponent({ name: 'DictationSettings' }).exists()).toBe(true);
    expect(w.findComponent({ name: 'ReadAloudSettings' }).exists()).toBe(true);
  });

  it('lists models and binary status in the dictation block', async () => {
    stubInvoke({
      list_voice_models: [{ id: 'ggml-base', sizeMb: 142, recommended: true, installed: false }],
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Whisper instalado');
    expect(w.text()).toContain('ggml-base');
    expect(w.text()).toContain('Descargar');
  });

  it('shows ASR model selector with installed voice models', async () => {
    stubInvoke({
      list_voice_models: [
        { id: 'ggml-base', sizeMb: 142, recommended: true, installed: true },
        { id: 'ggml-small', sizeMb: 466, recommended: false, installed: true },
      ],
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Selector de modelo ASR');
    expect(w.text()).toContain('Modelo de dictado');
    expect(w.text()).not.toContain('No instalado');
  });

  it('shows "not installed" badge when selected ASR model is not in installed list', async () => {
    stubInvoke({
      list_voice_models: [{ id: 'ggml-base', sizeMb: 142, recommended: true, installed: true }],
    });
    const { useVoiceSettingsStore } = await import('@/stores/voiceSettings');
    const store = useVoiceSettingsStore();
    store.asrModelId = 'ggml-large';
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('No instalado');
  });

  it('shows disk usage summary line when models are installed', async () => {
    stubInvoke({
      get_voice_status: { binaryInstalled: true, piperInstalled: true },
      list_voice_models: [{ id: 'ggml-base', sizeMb: 142, recommended: true, installed: true }],
      get_disk_usage: [
        { id: 'ggml-base', bytes: 148_897_792 },
        { id: 'es_ES-carlfm', bytes: 44_040_192 },
      ],
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Datos de voz');
    expect(w.text()).toMatch(/2 modelos/);
  });

  it('shows "no voice data" when disk is empty', async () => {
    stubInvoke();
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Sin datos de voz en disco');
  });

  it('hosts the dynamic voice catalog in the read-aloud block', async () => {
    stubInvoke({
      get_voice_status: { binaryInstalled: true, piperInstalled: true },
      get_voice_catalog: [
        {
          lang: 'es',
          langName: 'Español',
          featured: true,
          voices: [
            {
              id: 'es_ES-carlfm',
              name: 'Carl (FM)',
              lang: 'es',
              quality: 'medium',
              sizeMb: 42,
              recommended: true,
              installed: false,
            },
          ],
        },
      ],
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.findComponent({ name: 'VoiceCatalog' }).exists()).toBe(true);
    expect(w.text()).toContain('Carl (FM)');
    expect(w.text()).toContain('Piper instalado');
  });

  it('shows ASR test recorder section', async () => {
    stubInvoke();
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Probar dictado');
    expect(w.text()).toContain('Grabar prueba');
  });

  it('shows the detected acceleration backend', async () => {
    stubInvoke({
      get_accel_status: { backend: 'vulkan', model: 'small', serverAvailable: true },
    });
    const w = mountVoice();
    await flushPromises();
    expect(w.text()).toContain('Aceleración');
    expect(w.text()).toContain('Vulkan');
  });
});
