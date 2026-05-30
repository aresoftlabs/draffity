import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import AppBreadcrumb from './AppBreadcrumb.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: { es: { topBar: { breadcrumb: 'Navegación del proyecto' } } },
});

function mountCrumb(props: Record<string, unknown>) {
  return mount(AppBreadcrumb, { props, global: { plugins: [i18n] } });
}

describe('AppBreadcrumb', () => {
  it('no renderiza nada sin proyecto', () => {
    const wrapper = mountCrumb({ projectName: null, docTitle: null });
    expect(wrapper.find('nav').exists()).toBe(false);
  });

  it('muestra solo el proyecto cuando no hay documento', () => {
    const wrapper = mountCrumb({ projectName: 'El faro', docTitle: null });
    expect(wrapper.text()).toContain('El faro');
    expect(wrapper.find('[data-test="crumb-doc"]').exists()).toBe(false);
  });

  it('muestra proyecto › documento cuando hay ambos', () => {
    const wrapper = mountCrumb({ projectName: 'El faro', docTitle: 'Capítulo 3' });
    expect(wrapper.text()).toContain('El faro');
    expect(wrapper.find('[data-test="crumb-doc"]').text()).toContain('Capítulo 3');
  });
});
