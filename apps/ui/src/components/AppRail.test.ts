import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import type { ProjectViewMode } from '@/stores/ui';
import AppRail from './AppRail.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      viewMode: { editor: 'Manuscrito', corkboard: 'Corcho', outliner: 'Esquema', codex: 'Codex' },
      rail: { label: 'Actividad', search: 'Buscar' },
    },
  },
});

function mountRail(modelValue: ProjectViewMode = 'editor') {
  return mount(AppRail, { props: { modelValue }, global: { plugins: [i18n] } });
}

describe('AppRail', () => {
  it('emite update:modelValue al clickear otra vista', async () => {
    const w = mountRail('editor');
    await w.get('[aria-label="Corcho"]').trigger('click');
    expect(w.emitted('update:modelValue')?.[0]).toEqual(['corkboard']);
  });

  it('marca la vista activa con aria-current', () => {
    const w = mountRail('outliner');
    const active = w.get('[aria-current="page"]');
    expect(active.attributes('aria-label')).toBe('Esquema');
  });

  it('emite search desde su botón', async () => {
    const w = mountRail('editor');
    await w.get('[aria-label="Buscar"]').trigger('click');
    expect(w.emitted('search')).toBeTruthy();
  });
});
