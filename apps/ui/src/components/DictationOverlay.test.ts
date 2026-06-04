import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import DictationOverlay from './DictationOverlay.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      voice: {
        dictation: {
          recording: 'Grabando',
          transcribing: 'Transcribiendo',
          stop: 'Detener',
          cancel: 'Cancelar',
        },
      },
    },
  },
});

function mountOverlay(props: Record<string, unknown>) {
  return mount(DictationOverlay, {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    props: props as any,
    global: { plugins: [i18n, PrimeVue], directives: { tooltip: {} } },
  });
}

describe('DictationOverlay', () => {
  it('shows a determinate progress bar while transcribing with a progress value', () => {
    const w = mountOverlay({ phase: 'transcribing', level: 0, progress: 42 });
    const bar = w.find('[data-test="transcribe-progress"]');
    expect(bar.exists()).toBe(true);
    expect(bar.attributes('style')).toContain('42%');
    expect(w.text()).toContain('42');
  });

  it('falls back to the spinner when progress is null', () => {
    const w = mountOverlay({ phase: 'transcribing', level: 0, progress: null });
    expect(w.find('[data-test="transcribe-progress"]').exists()).toBe(false);
    expect(w.find('.pi-spinner').exists()).toBe(true);
  });
});
