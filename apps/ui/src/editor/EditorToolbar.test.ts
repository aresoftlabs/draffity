import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import EditorToolbar from './EditorToolbar.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  missingWarn: false,
  fallbackWarn: false,
  messages: { es: { toolbar: { dictate: 'Dictar' } } },
});

function mountToolbar(props: Record<string, unknown>) {
  return mount(EditorToolbar, {
    props: { editor: null, ...props } as never,
    global: { plugins: [i18n, PrimeVue], directives: { tooltip: {} } },
  });
}

describe('EditorToolbar dictation button', () => {
  it('emits toggle-dictation when the mic is clicked', async () => {
    const w = mountToolbar({ dictationAvailable: true });
    await w.find('[data-test="toolbar-dictate"]').trigger('click');
    expect(w.emitted('toggle-dictation')).toHaveLength(1);
  });

  it('disables the mic when dictation is unavailable', () => {
    const w = mountToolbar({ dictationAvailable: false });
    expect(w.find('[data-test="toolbar-dictate"]').attributes('disabled')).toBeDefined();
  });
});
