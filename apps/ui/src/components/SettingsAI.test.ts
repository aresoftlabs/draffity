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
        aiModelSelect: 'Modelo de IA',
        aiModelUnset: 'Seleccionar un modelo',
        aiModelNone: 'Sin modelos disponibles',
        aiModelAllLink: 'Ver todos en openrouter.ai/models',
        aiModelTokens: '{ctx} de contexto',
        aiModelCost: '~{cost}/M tokens',
        aiSpendingLimit: 'Límite mensual ($)',
        aiSpendingInvalid: 'Ingresá un monto válido',
        aiBudgetRemaining: '{used} de {limit} usados',
        aiBudgetOver: 'Excede el límite por {excess}',
        aiUsageLabel: 'Este mes: {sent} enviados / {received} recibidos',
      },
    },
  },
});

// The variable is whether a key is already stored.
function mockEnabled(hasKey: boolean) {
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === 'get_ai_status') return Promise.resolve({ available: true, hasKey });
    if (cmd === 'list_ai_models')
      return Promise.resolve([
        {
          id: 'gpt-4o',
          name: 'GPT-4o',
          provider: 'OpenAI',
          contextLength: 128000,
          costPer1kTokens: 0.005,
        },
        {
          id: 'claude-3-opus',
          name: 'Claude 3 Opus',
          provider: 'Anthropic',
          contextLength: 200000,
          costPer1kTokens: 0.015,
        },
        {
          id: 'gemini-1.5-pro',
          name: 'Gemini 1.5 Pro',
          provider: 'Google',
          contextLength: 1000000,
          costPer1kTokens: 0.0035,
        },
      ]);
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

  it('shows AI model selector with models grouped by provider', async () => {
    mockEnabled(true);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('Modelo de IA');
    // PrimeVue Select renders options lazily (not in DOM until opened),
    // so verify the Select component has the right options data.
    const select = w.findComponent({ name: 'Select' });
    expect(select.exists()).toBe(true);
    // @ts-expect-error — options is a Vue prop
    const opts = select.props('options');
    expect(opts).toHaveLength(3); // 3 providers
    expect(opts[0].label).toBe('OpenAI');
    expect(opts[1].label).toBe('Anthropic');
    expect(opts[2].label).toBe('Google');
  });

  it('shows "Select a model" placeholder when no AI model is selected', async () => {
    mockEnabled(true);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('Seleccionar un modelo');
  });

  it('shows spending limit input', async () => {
    mockEnabled(true);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('Límite mensual ($)');
  });

  it('shows token usage display', async () => {
    mockEnabled(true);
    const w = mountAI();
    await flushPromises();
    expect(w.text()).toContain('enviados');
  });
});
