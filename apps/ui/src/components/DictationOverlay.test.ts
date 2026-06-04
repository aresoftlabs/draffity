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
          silent: 'No se detecta voz',
        },
      },
    },
  },
});

function mountOverlay(props: Record<string, unknown>) {
  return mount(DictationOverlay, {
    props: props as never,
    global: { plugins: [i18n, PrimeVue], directives: { tooltip: {} } },
  });
}

describe('DictationOverlay', () => {
  it('is hidden when idle', () => {
    const w = mountOverlay({
      phase: 'idle',
      waveform: new Uint8Array(0),
      elapsedMs: 0,
      isSilent: false,
    });
    expect(w.find('[role="status"]').exists()).toBe(false);
  });

  it('shows the recording pill with the elapsed timer', () => {
    const w = mountOverlay({
      phase: 'recording',
      waveform: new Uint8Array([128]),
      elapsedMs: 7000,
      isSilent: false,
    });
    expect(w.text()).toContain('0:07');
  });

  it('shows a determinate progress bar while transcribing', () => {
    const w = mountOverlay({
      phase: 'transcribing',
      waveform: new Uint8Array(0),
      elapsedMs: 0,
      isSilent: false,
      progress: 42,
    });
    expect(w.find('[data-test="rec-progress"]').attributes('style')).toContain('42%');
  });
});
