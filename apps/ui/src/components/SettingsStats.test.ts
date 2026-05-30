import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import SettingsStats from './SettingsStats.vue';

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        writingStats: 'Estadísticas',
        currentStreak: 'Racha actual',
        longestStreak: 'Racha máxima',
        goalMetStreak: 'Racha de meta',
        lastWritingDate: 'Última actividad',
        dailyGoal: 'Meta diaria',
        dailyGoalNone: 'Sin meta',
        dailyGoalHint: 'hint',
        dailyGoalError: 'error',
        last30Days: 'Últimos 30 días',
        last30DaysAria: 'serie',
        totalWords: '{count} palabras',
        activeDays: '{count} días',
      },
    },
  },
});

function mountStats() {
  return mount(SettingsStats, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

describe('SettingsStats', () => {
  beforeEach(() => invokeMock.mockReset());

  it('renders the streak counters from the backend', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'get_writing_stats')
        return Promise.resolve({
          currentStreak: 5,
          longestStreak: 12,
          goalMetStreak: 3,
          lastWritingDate: '2026-05-30',
        });
      if (cmd === 'get_recent_daily_writing') return Promise.resolve([]);
      if (cmd === 'get_daily_goal') return Promise.resolve(500);
      return Promise.resolve(undefined);
    });
    const w = mountStats();
    await flushPromises();
    expect(w.text()).toContain('Estadísticas');
    expect(w.text()).toContain('Racha actual');
    expect(w.text()).toContain('5');
    expect(w.text()).toContain('12');
  });
});
