<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import Button from 'primevue/button';
import { useUiStore } from '@/stores/ui';
import { useDocumentStore } from '@/stores/document';
import { useProjectStore } from '@/stores/project';
import GoalProgress from '@/components/GoalProgress.vue';
import PomodoroWidget from '@/components/PomodoroWidget.vue';
import AppBreadcrumb from '@/components/AppBreadcrumb.vue';
import { useCommandPalette } from '@/composables/useCommandPalette';

const { t } = useI18n();
const router = useRouter();
const ui = useUiStore();
const docs = useDocumentStore();
const projects = useProjectStore();
const palette = useCommandPalette();

const sessionActive = computed(() => ui.sessionStartTotal !== null && docs.documents.length > 0);
const sessionWords = computed(() =>
  ui.sessionStartTotal === null ? 0 : Math.max(0, docs.totalWordCount - ui.sessionStartTotal),
);

function isDark() {
  return (
    ui.theme === 'dark' ||
    (ui.theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  );
}

function cycleTheme() {
  ui.cycleTheme();
}
</script>

<template>
  <header
    class="h-12 flex items-center px-4 gap-3 border-b border-surface-200/70 dark:border-surface-700/70 bg-surface-0/85 dark:bg-surface-950/85 backdrop-blur sticky top-0 z-10 shadow-[0_1px_3px_rgba(90,70,40,0.04)]"
  >
    <button
      class="font-display font-semibold tracking-tight text-surface-900 dark:text-surface-50"
      :title="t('app.tagline')"
      @click="router.push('/')"
    >
      {{ t('app.name') }}
    </button>

    <AppBreadcrumb
      :project-name="projects.current?.title ?? null"
      :doc-title="docs.selected?.title ?? null"
    />

    <span class="flex-1" />

    <div
      v-if="sessionActive"
      class="hidden md:flex items-center gap-2 min-w-[10rem] max-w-[18rem] text-xs opacity-90"
      :title="t('session.tooltip')"
    >
      <span class="opacity-60 shrink-0">{{ t('session.label') }}</span>
      <GoalProgress
        :current="sessionWords"
        :goal="ui.sessionGoal"
        compact
        @update:goal="ui.setSessionGoal"
      />
    </div>

    <PomodoroWidget />

    <Button
      type="button"
      severity="secondary"
      text
      class="!gap-2"
      :aria-label="t('commandPalette.open')"
      @click="palette.open()"
    >
      <i class="pi pi-search" />
      <span class="hidden sm:inline text-xs opacity-70">⌘K</span>
    </Button>

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
