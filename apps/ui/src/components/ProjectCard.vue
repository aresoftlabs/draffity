<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import type { Project } from '@draffity/shared-types';

defineProps<{
  project: Project;
  highlighted?: boolean;
}>();

const emit = defineEmits<{
  open: [id: string];
  delete: [id: string];
}>();

const { t } = useI18n();
</script>

<template>
  <article
    class="flex flex-col gap-3 p-5 rounded-lg border bg-surface-0 dark:bg-surface-900 transition-shadow hover:shadow-md"
    :class="
      highlighted
        ? 'border-primary-500 shadow-md ring-1 ring-primary-200 dark:ring-primary-900'
        : 'border-surface-200 dark:border-surface-700'
    "
  >
    <header class="flex items-start justify-between gap-2">
      <h3 class="text-lg font-semibold leading-tight font-serif truncate">
        {{ project.title }}
      </h3>
      <Tag
        :severity="project.status === 'active' ? 'success' : 'secondary'"
        :value="project.status === 'active' ? t('dashboard.active') : t('dashboard.readOnly')"
      />
    </header>

    <p class="text-xs opacity-60 font-mono">
      {{ project.templateId }}
    </p>

    <footer class="flex items-center justify-between mt-auto pt-2">
      <Button
        :label="project.status === 'active' ? t('dashboard.openProject') : t('dashboard.activate')"
        :icon="project.status === 'active' ? 'pi pi-arrow-right' : 'pi pi-play'"
        size="small"
        @click="emit('open', project.id)"
      />
      <Button
        icon="pi pi-trash"
        text
        severity="secondary"
        size="small"
        :aria-label="t('actions.delete')"
        @click="emit('delete', project.id)"
      />
    </footer>
  </article>
</template>
