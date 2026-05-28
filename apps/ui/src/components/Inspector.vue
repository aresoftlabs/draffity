<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { DocNode } from '@draffity/shared-types';
import SnapshotsList from '@/components/SnapshotsList.vue';

const props = defineProps<{
  doc: DocNode | null;
  wordCountHere: number;
  wordCountTotal: number;
  sessionWordCount?: number;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  snapshotRestored: [];
}>();

const { t, d, locale } = useI18n();

function formatDate(ts: number) {
  // Avoid throwing on weird timestamps.
  try {
    return d(new Date(ts), 'short');
  } catch {
    return new Date(ts).toLocaleString(locale.value);
  }
}

const docTypeLabel = computed(() => (props.doc ? t(`documentType.${props.doc.docType}`) : ''));
</script>

<template>
  <aside
    class="flex flex-col h-full bg-surface-50 dark:bg-surface-900 border-l border-surface-200 dark:border-surface-700"
  >
    <header class="px-3 py-2 border-b border-surface-200 dark:border-surface-700">
      <h3 class="text-sm font-semibold uppercase tracking-wide opacity-70">
        {{ t('project.inspector') }}
      </h3>
    </header>

    <div v-if="!doc" class="p-4 text-sm opacity-60">
      {{ t('project.noSelection') }}
    </div>

    <div v-else class="p-4 space-y-4 overflow-auto">
      <section>
        <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
          {{ t('project.metadata') }}
        </h4>
        <dl class="text-sm space-y-1">
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.type') }}</dt>
            <dd class="font-medium">{{ docTypeLabel }}</dd>
          </div>
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.createdAt') }}</dt>
            <dd>{{ formatDate(doc.createdAt) }}</dd>
          </div>
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.updatedAt') }}</dt>
            <dd>{{ formatDate(doc.updatedAt) }}</dd>
          </div>
        </dl>
      </section>

      <section>
        <dl class="text-sm space-y-1">
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.wordsHere') }}</dt>
            <dd class="font-mono">{{ wordCountHere }}</dd>
          </div>
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.wordsTotal') }}</dt>
            <dd class="font-mono">{{ wordCountTotal }}</dd>
          </div>
          <div v-if="sessionWordCount !== undefined" class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('project.wordsSession') }}</dt>
            <dd class="font-mono">{{ sessionWordCount }}</dd>
          </div>
        </dl>
      </section>

      <SnapshotsList
        :document-id="doc.id"
        :read-only="readOnly"
        @restored="emit('snapshotRestored')"
      />

      <section v-if="readOnly" class="text-xs opacity-70 italic">
        {{ t('project.readOnlyBanner') }}
      </section>
    </div>
  </aside>
</template>
