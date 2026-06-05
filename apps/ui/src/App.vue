<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue';
import { RouterView, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import Toast from 'primevue/toast';
import ConfirmDialog from 'primevue/confirmdialog';
import AppTopBar from '@/components/AppTopBar.vue';
import OnboardingDialog from '@/components/OnboardingDialog.vue';
import CommandPalette from '@/components/CommandPalette.vue';
import UpdateBanner from '@/components/UpdateBanner.vue';
import { useUpdater } from '@/composables/useUpdater';
import { registerCommands } from '@/composables/useCommandRegistry';
import { useCommandPalette } from '@/composables/useCommandPalette';
import { useShortcuts } from '@/composables/useShortcuts';
import { useUiStore } from '@/stores/ui';

const router = useRouter();
const { t } = useI18n();
const palette = useCommandPalette();
const ui = useUiStore();
const updater = useUpdater();

// Atajo global âŒ˜K (la acciÃ³n `commandPalette` ya existe en keybindings).
useShortcuts({ commandPalette: () => palette.toggle() });

// Comandos globales: disponibles en cualquier pantalla.
let offGlobalCmds: (() => void) | null = null;
onMounted(() => {
  void updater.check({ silent: true });
  offGlobalCmds = registerCommands([
    {
      id: 'global.dashboard',
      label: t('command.goDashboard'),
      group: t('command.groupGlobal'),
      icon: 'pi pi-home',
      run: () => void router.push('/'),
    },
    {
      id: 'global.settings',
      label: t('command.openSettings'),
      group: t('command.groupGlobal'),
      icon: 'pi pi-cog',
      run: () => void router.push('/settings'),
    },
    {
      id: 'global.theme',
      label: t('command.toggleTheme'),
      group: t('command.groupGlobal'),
      icon: 'pi pi-moon',
      keywords: ['tema', 'theme', 'oscuro', 'dark'],
      run: () => ui.toggleLightDark(),
    },
    {
      id: 'global.newDraft',
      label: t('command.newDraft'),
      group: t('command.groupGlobal'),
      icon: 'pi pi-plus',
      keywords: ['nuevo', 'new', 'draft', 'proyecto', 'project'],
      run: () => {
        // requestNewProject sets a one-shot flag that Dashboard reads in
        // onMounted and opens NewProjectWizard automatically.
        ui.requestNewProject();
        void router.push('/');
      },
    },
  ]);
});
onBeforeUnmount(() => offGlobalCmds?.());
</script>

<template>
  <div
    class="draffity-app flex flex-col h-screen overflow-hidden bg-surface-50 dark:bg-surface-950 text-surface-900 dark:text-surface-50"
  >
    <AppTopBar v-if="!ui.compositionMode" />
    <main class="flex-1 flex flex-col min-h-0 overflow-hidden">
      <RouterView />
    </main>
    <OnboardingDialog />
    <Toast position="bottom-right" />
    <ConfirmDialog />
    <CommandPalette />
    <UpdateBanner />
  </div>
</template>
