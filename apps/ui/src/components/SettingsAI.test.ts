import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import { createPinia, setActivePinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import SettingsAI from './SettingsAI.vue';

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        aiTitle: 'IA',
        aiKeyHint: 'hint',
        aiKeySaved: 'Clave guardada',
        aiKeyClear: 'Quitar clave',
        aiKeyPlaceholder: 'sk-...',
        aiKeyLabel: 'Clave',
        aiKeySave: 'Guardar clave',
        aiKeyGetLink: 'Conseguir clave',
        aiUsageThisMonth: '{sent} enviados · {received} recibidos',
        aiUsageReset: 'Reiniciar',
        aiUsageCostsLink: 'Ver costos',
        loadError: 'No se pudo cargar',
      },
      capability: { unavailable: 'No disponible' },
    },
  },
});

// The variable is whether a key is already stored.
function mockEnabled(hasKey: boolean) {
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === 'get_ai_status') return Promise.resolve({ available: true, hasKey });
    return Promise.resolve(undefined);
  });
}

function mountAI() {
  return mount(SettingsAI, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

describe('SettingsAI', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it('shows the key-entry UI when no key is stored', async () => {
    mockEnabled(false);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('Guardar clave');
    expect(w.text()).not.toContain('No disponible');
  });

  it('shows the stored-key state when a key already exists', async () => {
    mockEnabled(true);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('Clave guardada');
    expect(w.text()).toContain('Quitar clave');
  });
});
