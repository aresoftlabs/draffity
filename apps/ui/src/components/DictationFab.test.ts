import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import DictationFab from './DictationFab.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: { es: { toolbar: { dictate: 'Dictar' } } },
});

function mountFab(props: Record<string, unknown>) {
  return mount(DictationFab, {
    props: props as never,
    global: { plugins: [i18n, PrimeVue], directives: { tooltip: {} } },
  });
}

describe('DictationFab', () => {
  it('is hidden when dictation is unavailable', () => {
    const w = mountFab({ available: false, active: false });
    expect(w.find('[data-test="dictation-fab"]').exists()).toBe(false);
  });

  it('is hidden while a recording/transcription is active (the pill takes over)', () => {
    const w = mountFab({ available: true, active: true });
    expect(w.find('[data-test="dictation-fab"]').exists()).toBe(false);
  });

  it('shows and emits toggle when available and idle', async () => {
    const w = mountFab({ available: true, active: false });
    const btn = w.find('[data-test="dictation-fab"]');
    expect(btn.exists()).toBe(true);
    await btn.trigger('click');
    expect(w.emitted('toggle')).toHaveLength(1);
  });
});
