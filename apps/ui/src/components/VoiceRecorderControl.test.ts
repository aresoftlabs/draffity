import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import VoiceRecorderControl from './VoiceRecorderControl.vue';

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

function mountControl(props: Record<string, unknown>) {
  return mount(VoiceRecorderControl, {
    props: props as never,
    global: { plugins: [i18n, PrimeVue], directives: { tooltip: {} } },
  });
}

describe('VoiceRecorderControl', () => {
  it('shows elapsed timer and emits stop/cancel while recording', async () => {
    const w = mountControl({
      state: 'recording',
      waveform: new Uint8Array([128]),
      elapsedMs: 7000,
      isSilent: false,
      progress: null,
    });
    expect(w.text()).toContain('0:07');
    await w.find('[data-test="rec-stop"]').trigger('click');
    await w.find('[data-test="rec-cancel"]').trigger('click');
    expect(w.emitted('stop')).toHaveLength(1);
    expect(w.emitted('cancel')).toHaveLength(1);
  });

  it('shows the silence warning when isSilent', () => {
    const w = mountControl({
      state: 'recording',
      waveform: new Uint8Array([128]),
      elapsedMs: 3000,
      isSilent: true,
      progress: null,
    });
    expect(w.text()).toContain('No se detecta voz');
  });

  it('shows a determinate progress bar while transcribing', () => {
    const w = mountControl({
      state: 'transcribing',
      waveform: new Uint8Array(0),
      elapsedMs: 0,
      isSilent: false,
      progress: 42,
    });
    const bar = w.find('[data-test="rec-progress"]');
    expect(bar.exists()).toBe(true);
    expect(bar.attributes('style')).toContain('42%');
  });
});
