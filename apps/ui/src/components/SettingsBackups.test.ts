import { beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import { invoke } from '@tauri-apps/api/core';
import SettingsBackups from './SettingsBackups.vue';

const invokeMock = vi.mocked(invoke);

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      settings: {
        backupsTitle: 'Copias',
        backupsHint: 'hint',
        backupNow: 'Crear copia',
        backupsEmpty: 'Sin copias todavía',
        restore: 'Restaurar',
        backupsError: 'error',
        backupCreated: 'creada',
        restoreSuccess: 'restaurada',
        restoreConfirm: 'confirmar',
        backupKind: { manual: 'Manual', daily: 'Diaria', monthly: 'Mensual' },
      },
    },
  },
});

function mountBackups() {
  return mount(SettingsBackups, { global: { plugins: [i18n, PrimeVue, ToastService] } });
}

describe('SettingsBackups', () => {
  beforeEach(() => invokeMock.mockReset());

  it('shows the empty state when the backend returns no backups', async () => {
    invokeMock.mockResolvedValueOnce([]); // list_backups
    const w = mountBackups();
    await flushPromises();
    expect(w.text()).toContain('Sin copias todavía');
  });

  it('lists the backups returned by the backend', async () => {
    invokeMock.mockResolvedValueOnce([
      { id: '2026-05-30-manual', path: '/x', createdAt: 0, sizeBytes: 2048, kind: 'manual' },
    ]);
    const w = mountBackups();
    await flushPromises();
    expect(w.text()).toContain('2026-05-30-manual');
    expect(w.text()).toContain('Manual');
    expect(w.text()).toContain('2.0 KB');
  });
});
