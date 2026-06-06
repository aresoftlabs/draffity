import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import { createPinia, setActivePinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import DictationSettings from './DictationSettings.vue';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        voiceTitle: 'Voz',
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
        voiceModelSelector: 'Selector de modelo ASR',
        voiceModelSelectorLabel: 'Modelo de dictado',
        voiceModelNotInstalled: 'No instalado',
        voiceDiskLabel: 'En disco',
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
        voiceAutoStopSilence: 'Detener tras silencio',
        voiceAutoStopSilenceHint: 'Finaliza la grabación sola tras silencio.',
        voiceAccel: 'Aceleración',
        voiceAccelModel: 'Modelo activo',
        voiceAccelRedetect: 'Re-detectar',
        voiceAccelMetal: 'Metal (GPU Apple)',
        voiceAccelVulkan: 'Vulkan (GPU)',
        voiceAccelCpu: 'CPU',
        voiceDictationMode: 'Modo de dictado en vivo',
        voiceDictationModeHint: 'Cuando está activo, el texto aparece mientras hablás.',
      },
    },
  },
});

function stubInvoke(over: Partial<Record<string, unknown>> = {}) {
  const defaults: Record<string, unknown> = {
    list_voice_models: [],
    get_accel_status: null,
  };
  const table = { ...defaults, ...over };
  invokeMock.mockImplementation((cmd: string) =>
    cmd in table ? Promise.resolve(table[cmd]) : Promise.resolve(undefined),
  );
}

function mountComponent() {
  return mount(DictationSettings, {
    props: { voiceStatus: null, downloadPct: {} },
    global: { plugins: [i18n, PrimeVue, ToastService] },
  });
}

describe('DictationSettings', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
    Object.defineProperty(globalThis.navigator, 'mediaDevices', {
      value: {
        enumerateDevices: vi.fn().mockResolvedValue([]),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      },
      writable: true,
    });
  });

  it('renders the dictation section', async () => {
    stubInvoke();
    const w = mountComponent();
    await flushPromises();
    expect(w.find('[data-test="dictation-settings"]').exists()).toBe(true);
    expect(w.text()).toContain('Dictado');
  });

  it('dictation mode toggle flips store dictationMode to streaming', async () => {
    stubInvoke();
    const w = mountComponent();
    await flushPromises();

    const store = useVoiceSettingsStore();
    expect(store.dictationMode).toBe('manual');

    // Find the ToggleSwitch bound to dictation-mode by its inputId prop.
    const modeToggle = w
      .findAllComponents({ name: 'ToggleSwitch' })
      .find((c) => c.props('inputId') === 'dictation-mode');

    expect(modeToggle).toBeDefined();
    // Invoke the bound update handler with true (streaming on).
    await modeToggle!.vm.$emit('update:modelValue', true);

    expect(store.dictationMode).toBe('streaming');
  });
});
