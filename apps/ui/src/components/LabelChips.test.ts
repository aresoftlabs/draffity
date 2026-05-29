import { beforeEach, describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';

import LabelChips from './LabelChips.vue';
import { useLabelStore } from '@/stores/labels';
import type { Label } from '@draffity/shared-types';

function label(id: string, name: string, color = '#ef4444'): Label {
  return { id, projectId: 'p1', name, color, createdAt: 0 };
}

describe('LabelChips', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    const store = useLabelStore();
    store.labels = [label('a', 'Alfa'), label('b', 'Beta', '#3b82f6'), label('c', 'Gamma')];
  });

  it('renders a chip per resolvable id, skipping unknown ids', () => {
    const w = mount(LabelChips, { props: { labelIds: ['a', 'zzz', 'b'] } });
    const text = w.text();
    expect(text).toContain('Alfa');
    expect(text).toContain('Beta');
    // Unknown id contributes nothing (each chip is an inline-flex pill).
    expect(w.findAll('span.inline-flex').length).toBe(2);
  });

  it('renders nothing when no id resolves', () => {
    const w = mount(LabelChips, { props: { labelIds: ['nope'] } });
    expect(w.find('div').exists()).toBe(false);
  });

  it('collapses overflow beyond max into a +N counter', () => {
    const w = mount(LabelChips, { props: { labelIds: ['a', 'b', 'c'], max: 2 } });
    expect(w.text()).toContain('+1');
    expect(w.text()).toContain('Alfa');
    expect(w.text()).toContain('Beta');
    expect(w.text()).not.toContain('Gamma');
  });
});
