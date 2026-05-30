import { describe, expect, it, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { setActivePinia, createPinia } from 'pinia';
import { createI18n } from 'vue-i18n';

// The palette reads live keybindings via the store; stub the backend so
// `keybindings.load()` resolves without a Tauri runtime.
vi.mock('@/services/ipc', () => ({
  ipc: {
    getSetting: vi.fn().mockResolvedValue(null),
    setSetting: vi.fn().mockResolvedValue(undefined),
  },
}));

import CommandPalette from './CommandPalette.vue';
import { useCommandRegistry, registerCommands } from '@/composables/useCommandRegistry';
import { useCommandPalette } from '@/composables/useCommandPalette';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      commandPalette: {
        placeholder: 'Escribí un comando o buscá…',
        noResults: 'Sin comandos coincidentes',
        recent: 'Recientes',
        hintNavigate: '↑↓',
        hintRun: '↵',
        hintClose: 'esc',
        open: 'Paleta de comandos',
      },
    },
  },
});

function mountPalette() {
  return mount(CommandPalette, {
    global: {
      plugins: [i18n],
      // PrimeVue Dialog usa Teleport; stub para render inline en el test.
      stubs: { Dialog: { template: '<div v-if="visible"><slot /></div>', props: ['visible'] } },
    },
  });
}

describe('CommandPalette', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    useCommandRegistry().clearAll();
    useCommandPalette().close();
  });

  it('filtra los comandos según el texto tipeado', async () => {
    const ran = vi.fn();
    registerCommands([
      { id: 'export', label: 'Exportar', group: 'Proyecto', run: ran },
      { id: 'focus', label: 'Modo foco', group: 'Proyecto', run: () => {} },
    ]);
    useCommandPalette().open();
    const wrapper = mountPalette();
    await wrapper.vm.$nextTick();

    await wrapper.find('input').setValue('export');
    const items = wrapper.findAll('[data-test="command-item"]');
    expect(items.length).toBe(1);
    expect(items[0].text()).toContain('Exportar');
  });

  it('ejecuta el comando resaltado al presionar Enter y cierra la paleta', async () => {
    const ran = vi.fn();
    registerCommands([{ id: 'export', label: 'Exportar', group: 'Proyecto', run: ran }]);
    useCommandPalette().open();
    const wrapper = mountPalette();
    await wrapper.vm.$nextTick();

    await wrapper.find('input').trigger('keydown', { key: 'Enter' });
    expect(ran).toHaveBeenCalledOnce();
    expect(useCommandPalette().visible.value).toBe(false);
  });
});
