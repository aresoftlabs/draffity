<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { ProjectViewMode } from '@/stores/ui';

/** Riel de actividad presentacional. No accede a stores: recibe la vista activa
 *  y emite los cambios; el padre (ProjectView) persiste vía changeViewMode. */
defineProps<{ modelValue: ProjectViewMode }>();
const emit = defineEmits<{
  'update:modelValue': [ProjectViewMode];
  search: [];
  settings: [];
}>();

const { t } = useI18n();

const views: { mode: ProjectViewMode; icon: string; key: string }[] = [
  { mode: 'editor', icon: 'pi pi-pencil', key: 'viewMode.editor' },
  { mode: 'corkboard', icon: 'pi pi-th-large', key: 'viewMode.corkboard' },
  { mode: 'outliner', icon: 'pi pi-list', key: 'viewMode.outliner' },
  { mode: 'codex', icon: 'pi pi-book', key: 'viewMode.codex' },
];
</script>

<template>
  <nav
    class="w-[52px] shrink-0 flex flex-col items-center gap-1 py-3 border-r border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900"
    :aria-label="t('rail.label')"
  >
    <button
      v-for="v in views"
      :key="v.mode"
      type="button"
      class="w-9 h-9 rounded-[10px] flex items-center justify-center transition-colors"
      :class="
        modelValue === v.mode
          ? 'bg-primary-500 text-white shadow-sm'
          : 'text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800'
      "
      :title="t(v.key)"
      :aria-label="t(v.key)"
      :aria-current="modelValue === v.mode ? 'page' : undefined"
      @click="emit('update:modelValue', v.mode)"
    >
      <i :class="v.icon" />
    </button>

    <button
      type="button"
      class="w-9 h-9 rounded-[10px] flex items-center justify-center text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
      :title="t('rail.search')"
      :aria-label="t('rail.search')"
      @click="emit('search')"
    >
      <i class="pi pi-search" />
    </button>

    <button
      type="button"
      class="mt-auto w-9 h-9 rounded-[10px] flex items-center justify-center text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
      :title="t('settings.title')"
      :aria-label="t('settings.title')"
      @click="emit('settings')"
    >
      <i class="pi pi-cog" />
    </button>
  </nav>
</template>
