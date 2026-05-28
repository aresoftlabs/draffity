<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import Button from 'primevue/button';
import { useUiStore } from '@/stores/ui';

const { t } = useI18n();
const router = useRouter();
const ui = useUiStore();

function isDark() {
  return (
    ui.theme === 'dark' ||
    (ui.theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  );
}

function cycleTheme() {
  if (ui.theme === 'system') ui.setTheme('light');
  else if (ui.theme === 'light') ui.setTheme('dark');
  else ui.setTheme('system');
}
</script>

<template>
  <header
    class="h-12 flex items-center px-4 gap-3 border-b border-surface-200 dark:border-surface-700 bg-surface-0/80 dark:bg-surface-950/80 backdrop-blur sticky top-0 z-10"
  >
    <button
      class="font-serif font-semibold tracking-tight"
      :title="t('app.tagline')"
      @click="router.push('/')"
    >
      {{ t('app.name') }}
    </button>

    <span class="flex-1" />

    <Button
      :icon="isDark() ? 'pi pi-sun' : 'pi pi-moon'"
      text
      rounded
      severity="secondary"
      :aria-label="t('settings.theme')"
      @click="cycleTheme"
    />
    <Button
      icon="pi pi-cog"
      text
      rounded
      severity="secondary"
      :aria-label="t('settings.title')"
      @click="router.push('/settings')"
    />
  </header>
</template>
