<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import type { Project } from '@draffity/shared-types';
import { coverTone } from '@/components/projectCover';

const props = defineProps<{
  project: Project;
  highlighted?: boolean;
}>();

const emit = defineEmits<{
  open: [id: string];
  delete: [id: string];
}>();

const { t } = useI18n();

const tone = computed(() => coverTone(props.project.id));
const editedLabel = computed(() => new Date(props.project.updatedAt).toLocaleDateString());
</script>

<template>
  <article
    class="flex flex-col rounded-2xl border overflow-hidden bg-surface-0 dark:bg-surface-900 transition-shadow hover:shadow-md"
    :class="
      highlighted
        ? 'border-primary-500 shadow-md ring-1 ring-primary-200 dark:ring-primary-900'
        : 'border-surface-200 dark:border-surface-700'
    "
  >
    <div class="h-24 px-4 py-3 flex flex-col justify-between" :style="{ backgroundColor: tone }">
      <span class="text-[10px] uppercase tracking-[0.14em] font-medium text-[#5c4a2e]/70">
        {{ project.templateId }}
      </span>
      <h3 class="font-display font-semibold text-lg leading-tight text-[#3f3320] line-clamp-2">
        {{ project.title }}
      </h3>
    </div>

    <div
      class="flex items-center justify-between gap-2 px-4 py-2 border-b border-surface-100 dark:border-surface-800"
    >
      <span class="text-xs opacity-60">{{ t('dashboard.editedAt') }} {{ editedLabel }}</span>
      <Tag
        :severity="project.status === 'active' ? 'success' : 'secondary'"
        :value="project.status === 'active' ? t('dashboard.active') : t('dashboard.readOnly')"
      />
    </div>

    <footer class="flex items-center justify-between px-4 py-3">
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
