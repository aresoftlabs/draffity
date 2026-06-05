import { describe, expect, it, vi } from 'vitest';
import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';

interface UpdaterState {
  status: Ref<string>;
  availableVersion: Ref<string>;
  notes: Ref<string | null>;
  progress: Ref<number>;
  errorMessage: Ref<string | null>;
  bannerVisible: ComputedRef<boolean>;
  check: ReturnType<typeof vi.fn>;
  downloadAndInstall: ReturnType<typeof vi.fn>;
  relaunchApp: ReturnType<typeof vi.fn>;
  dismiss: ReturnType<typeof vi.fn>;
}

const state: UpdaterState = {
  status: ref('available'),
  availableVersion: ref('0.13.0'),
  notes: ref<string | null>('Novedades'),
  progress: ref(0),
  errorMessage: ref<string | null>(null),
  bannerVisible: computed(() => ['available', 'downloading', 'ready'].includes(state.status.value)),
  check: vi.fn(),
  downloadAndInstall: vi.fn(),
  relaunchApp: vi.fn(),
  dismiss: vi.fn(),
};
vi.mock('@/composables/useUpdater', () => ({ useUpdater: () => state }));

const i18n = createI18n({
  legacy: false,
  locale: 'es',
  messages: {
    es: {
      updater: {
        available: 'Draffity {version} disponible',
        updateNow: 'Actualizar ahora',
        later: 'Más tarde',
        downloading: 'Descargando… {percent}%',
        restart: 'Reiniciar para aplicar',
      },
    },
  },
});

async function mountBanner() {
  const UpdateBanner = (await import('./UpdateBanner.vue')).default;
  return mount(UpdateBanner, { global: { plugins: [i18n] } });
}

describe('UpdateBanner', () => {
  it('shows the available version and triggers download on "Actualizar ahora"', async () => {
    state.status.value = 'available';
    state.progress.value = 0;
    const w = await mountBanner();
    expect(w.text()).toContain('Draffity 0.13.0 disponible');
    await w.get('[data-test="update-now"]').trigger('click');
    expect(state.downloadAndInstall).toHaveBeenCalledOnce();
  });

  it('calls dismiss on "Más tarde"', async () => {
    state.status.value = 'available';
    const w = await mountBanner();
    await w.get('[data-test="update-later"]').trigger('click');
    expect(state.dismiss).toHaveBeenCalledOnce();
  });

  it('shows a restart button once ready', async () => {
    state.status.value = 'ready';
    const w = await mountBanner();
    await w.get('[data-test="update-restart"]').trigger('click');
    expect(state.relaunchApp).toHaveBeenCalledOnce();
  });
});
