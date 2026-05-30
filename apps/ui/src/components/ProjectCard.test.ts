import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import ProjectCard from './ProjectCard.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      dashboard: {
        active: 'Activo',
        readOnly: 'Solo lectura',
        openProject: 'Abrir',
        activate: 'Activar',
        editedAt: 'Editado',
      },
      actions: { delete: 'Eliminar' },
    },
  },
});

const project = {
  id: 'el-faro',
  title: 'El faro al final del invierno',
  templateId: 'novel',
  status: 'active' as const,
  createdAt: 1_700_000_000_000,
  updatedAt: 1_700_000_000_000,
};

function mountCard() {
  return mount(ProjectCard, {
    props: { project },
    global: { plugins: [i18n], stubs: { Button: true, Tag: true } },
  });
}

describe('ProjectCard', () => {
  it('muestra el título del proyecto', () => {
    expect(mountCard().text()).toContain('El faro al final del invierno');
  });

  it('declara el emit open con el id', () => {
    const wrapper = mountCard();
    (wrapper.vm as unknown as { $emit: (e: string, ...a: unknown[]) => void }).$emit(
      'open',
      'el-faro',
    );
    expect(wrapper.emitted('open')?.[0]).toEqual(['el-faro']);
  });
});
