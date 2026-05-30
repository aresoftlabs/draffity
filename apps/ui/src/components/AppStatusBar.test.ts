import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import AppStatusBar from './AppStatusBar.vue';

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      statusBar: { words: 'palabras', session: 'Sesión', goal: 'Objetivo' },
      save: { saving: 'Guardando…', saved: 'Guardado', error: 'Error', idle: '' },
      pomodoro: { title: 'Temporizador' },
    },
  },
});

const baseProps = {
  wordCount: 320,
  totalWordCount: 1240,
  saveState: 'saved' as const,
  lastSavedAt: null,
  projectGoal: 2000,
  projectDeadline: null,
  sessionWords: 120,
  sessionGoal: 500,
  readOnly: false,
};

function mountBar(extra: Record<string, unknown> = {}) {
  return mount(AppStatusBar, {
    props: { ...baseProps, ...extra },
    global: {
      plugins: [i18n],
      stubs: {
        SaveIndicator: true,
        PacemakerWidget: true,
        PomodoroWidget: true,
        GoalProgress: true,
      },
    },
  });
}

describe('AppStatusBar', () => {
  it('muestra el contador de palabras total', () => {
    const wrapper = mountBar();
    expect(wrapper.text()).toMatch(/1.?240/);
  });

  it('declara y reemite update:projectGoal', () => {
    const wrapper = mountBar();
    (wrapper.vm as unknown as { $emit: (e: string, ...a: unknown[]) => void }).$emit(
      'update:projectGoal',
      3000,
    );
    expect(wrapper.emitted('update:projectGoal')?.[0]).toEqual([3000]);
  });
});
