import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import VoiceCatalog from './VoiceCatalog.vue';
import type { CatalogLang } from '@/services/ipc';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        voiceCatalogSearch: 'Buscar voz',
        voiceCatalogAllLangs: 'Todos los idiomas ({count})',
        voiceTestPlay: 'Probar voz',
        voiceModelDownload: 'Descargar',
        voiceDelete: 'Eliminar',
      },
    },
  },
});

const catalog: CatalogLang[] = [
  {
    lang: 'es',
    langName: 'Español',
    featured: true,
    voices: [
      {
        id: 'es_ES-davefx-medium',
        name: 'Dave',
        lang: 'es',
        quality: 'medium',
        sizeMb: 63,
        recommended: true,
        installed: true,
      },
      {
        id: 'es_ES-sharvard-medium',
        name: 'Sharvard',
        lang: 'es',
        quality: 'medium',
        sizeMb: 60,
        recommended: false,
        installed: false,
      },
    ],
  },
  {
    lang: 'de',
    langName: 'Deutsch',
    featured: false,
    voices: [
      {
        id: 'de_DE-thorsten-medium',
        name: 'Thorsten',
        lang: 'de',
        quality: 'medium',
        sizeMb: 60,
        recommended: false,
        installed: false,
      },
    ],
  },
];

const globalOpts = { plugins: [i18n, PrimeVue, ToastService] };

describe('VoiceCatalog', () => {
  it('lists featured langs first and shows their voices', () => {
    const w = mount(VoiceCatalog, { props: { catalog, downloadPct: {} }, global: globalOpts });
    const text = w.text();
    expect(text).toContain('Español');
    expect(text).toContain('Dave');
    expect(text.indexOf('Español')).toBeLessThan(text.indexOf('Deutsch'));
  });

  it('emits delete for an installed voice', async () => {
    const w = mount(VoiceCatalog, { props: { catalog, downloadPct: {} }, global: globalOpts });
    await w.find('[data-test="delete-es_ES-davefx-medium"]').trigger('click');
    expect(w.emitted('delete')?.[0]).toEqual(['es_ES-davefx-medium']);
  });

  it('emits download for a non-installed voice', async () => {
    const w = mount(VoiceCatalog, { props: { catalog, downloadPct: {} }, global: globalOpts });
    await w.find('[data-test="download-es_ES-sharvard-medium"]').trigger('click');
    expect(w.emitted('download')?.[0]).toEqual(['es_ES-sharvard-medium']);
  });

  it('filters by search query (voice or language)', async () => {
    const w = mount(VoiceCatalog, { props: { catalog, downloadPct: {} }, global: globalOpts });
    await w.find('input').setValue('thorsten');
    expect(w.text()).toContain('Thorsten');
    expect(w.text()).not.toContain('Dave');
  });
});
