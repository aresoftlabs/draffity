<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { Template } from '@draffity/shared-types';

defineProps<{
  template: Template | null;
  title: string;
  values: Record<string, unknown>;
  nodeCount: number;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex flex-col gap-3 min-h-[20rem]">
    <h3 class="text-sm font-semibold uppercase tracking-wide opacity-60">
      {{ t('newProject.summary') }}
    </h3>
    <dl class="text-sm space-y-1">
      <div class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('newProject.name') }}</dt>
        <dd class="font-medium">{{ title.trim() }}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('newProject.template') }}</dt>
        <dd class="font-medium">{{ template?.name }}</dd>
      </div>
      <div
        v-for="f in template?.metadataFields ?? []"
        :key="f.key"
        class="flex justify-between gap-2"
      >
        <dt class="opacity-60">{{ f.label }}</dt>
        <dd>{{ values[f.key] || 'â€”' }}</dd>
      </div>
    </dl>

    <p class="text-xs opacity-70 italic mt-2">
      {{ t('newProject.willCreate', { count: nodeCount }) }}
    </p>
  </div>
</template>
