import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';

import CustomFieldsEditor from './CustomFieldsEditor.vue';
import type { CustomField } from '@draffity/shared-types';

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  missingWarn: false,
  fallbackWarn: false,
  messages: { en: {} },
});

function field(over: Partial<CustomField>): CustomField {
  return {
    id: over.id ?? 'f',
    projectId: 'p',
    name: over.name ?? 'Field',
    kind: over.kind ?? 'text',
    options: over.options ?? [],
    position: 0,
    createdAt: 0,
  };
}

function mountEditor(fields: CustomField[], values: Record<string, string> = {}) {
  return mount(CustomFieldsEditor, {
    props: { fields, values },
    global: { plugins: [i18n, [PrimeVue, { theme: 'none' }]] },
  });
}

describe('CustomFieldsEditor', () => {
  it('renders a labelled control per field', () => {
    const w = mountEditor([
      field({ id: 'a', name: 'Reviewer', kind: 'text' }),
      field({ id: 'b', name: 'POV', kind: 'select', options: ['Alice', 'Bob'] }),
    ]);
    expect(w.text()).toContain('Reviewer');
    expect(w.text()).toContain('POV');
  });

  it('emits a trimmed change on text blur, null when blank', async () => {
    const w = mountEditor([field({ id: 'a', kind: 'text' })]);
    const input = w.get('input');
    await input.setValue('  hi  ');
    await input.trigger('blur');
    await input.setValue('   ');
    await input.trigger('blur');

    const events = w.emitted('change');
    expect(events).toBeTruthy();
    expect(events![0]).toEqual(['a', 'hi']);
    expect(events![1]).toEqual(['a', null]);
  });
});
