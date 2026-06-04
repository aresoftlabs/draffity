import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import { createPinia, setActivePinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import Settings from './Settings.vue';

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
vi.mock('@tauri-apps/plugin-fs', () => ({ readFile: vi.fn() }));

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        title: 'Ajustes',
        nav: {
          appearance: 'Apariencia',
          editor: 'Editor',
          language: 'Idioma',
          audio: 'Audio',
          ai: 'IA',
          shortcuts: 'Atajos',
          goals: 'Objetivos',
          data: 'Datos',
          about: 'Acerca de',
        },
        resourcesPath: 'Carpeta de recursos',
        resourcesPathChange: 'Cambiar…',
        resourcesPathSaved: 'Ruta guardada',
        resourcesPathRestart: 'Reiniciar ahora',
        backupsTitle: 'Copias',
        theme: 'Tema',
        themeLight: 'Claro',
        themeDark: 'Oscuro',
        themeHighContrast: 'Alto contraste',
        themeSystem: 'Sistema',
        language: 'Idioma',
        editorFont: 'Fuente',
        fontSerif: 'Serif',
        fontSans: 'Sans',
        fontMono: 'Mono',
        fontGroupBuiltIn: 'Incluidas',
        fontGroupSystem: 'Sistema',
        fontPickerPlaceholder: 'Elige',
        fontHint: 'hint',
        typewriter: 'Typewriter',
        typewriterHint: 'hint',
        linguisticFocusTitle: 'Foco',
        linguisticFocusHint: 'hint',
        linguisticExtraPlaceholder: 'agregar',
        readingSpeed: 'Velocidad',
        readingSpeedHint: 'hint',
        autosave: 'Autoguardado',
        autosaveHint: 'hint',
        customCss: 'CSS',
        customCssHint: 'hint',
        customCssPlaceholder: 'css',
        uploadFont: 'Subir',
        uploadFontNoProject: 'Abrir proyecto',
        fontUploaded: 'Subida',
        fontUploadFailed: 'Error',
        backupsHint: 'hint',
        backupNow: 'Crear',
        backupsEmpty: 'Sin copias',
        restore: 'Restaurar',
        backupsError: 'error',
        backupCreated: 'creada',
        restoreSuccess: 'restaurada',
        restoreConfirm: 'confirmar',
        backupKind: { manual: 'Manual', daily: 'Diaria', monthly: 'Mensual' },
      },
    },
  },
});

function mountSettings() {
  return mount(Settings, {
    global: {
      plugins: [i18n, createPinia(), PrimeVue, ToastService],
      stubs: {
        KeybindingsEditor: true,
        LegalDialog: true,
        SettingsBackups: true,
        SettingsAI: true,
        SettingsVoice: true,
        SettingsStats: true,
        SelectButton: true,
        Select: true,
        Slider: true,
        ToggleSwitch: true,
        InputNumber: true,
        Chips: true,
        Textarea: true,
      },
    },
  });
}

describe('Settings — resources path', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('shows resources path section with path and change button when navigating to data tab', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'list_project_media') return Promise.resolve([]);
      if (cmd === 'get_crash_reporting_status')
        return Promise.resolve({ active: false, enabled: false });
      if (cmd === 'get_resources_path')
        return Promise.resolve('C:\\Users\\test\\Draffity\\resources');
      return Promise.resolve(undefined);
    });

    const w = mountSettings();
    await flushPromises();

    // Navigate to data section
    const dataBtn = w.findAll('button').find((b) => b.text().includes('Datos'));
    expect(dataBtn).toBeDefined();
    await dataBtn!.trigger('click');
    await flushPromises();

    expect(w.text()).toContain('Carpeta de recursos');
    expect(w.text()).toContain('C:\\Users\\test\\Draffity\\resources');
    expect(w.text()).toContain('Cambiar…');
  });
});
