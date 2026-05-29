<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Select from 'primevue/select';
import Chips from 'primevue/chips';
import Textarea from 'primevue/textarea';
import type { DocNode, DocumentStatus } from '@draffity/shared-types';
import SnapshotsList from '@/components/SnapshotsList.vue';
import GoalProgress from '@/components/GoalProgress.vue';

const props = defineProps<{
  doc: DocNode | null;
  wordCountHere: number;
  wordCountTotal: number;
  sessionWordCount?: number;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  snapshotRestored: [];
  statusChange: [status: DocumentStatus];
  tagsChange: [tags: string[]];
  goalChange: [goal: number | null];
  synopsisChange: [synopsis: string | null];
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

const statusOptions = computed<{ value: DocumentStatus; label: string }[]>(() =>
  (['draft', 'revised', 'final', 'trashed'] as const).map((value) => ({
    value,
    label: t(`status.${value}`),
  })),
);

function onStatusChange(value: DocumentStatus) {
  if (!props.doc || props.readOnly) return;
  if (value === props.doc.status) return;
  emit('statusChange', value);
}

function onTagsChange(next: string[]) {
  if (!props.doc || props.readOnly) return;
  // Normalise: trim + dedupe + drop empties so the UI mirrors what the
  // backend stores. Order is preserved.
  const seen = new Set<string>();
  const cleaned: string[] = [];
  for (const raw of next) {
    const t = String(raw).trim();
    if (!t || seen.has(t)) continue;
    seen.add(t);
    cleaned.push(t);
  }
  emit('tagsChange', cleaned);
}

let synopsisDebounce: ReturnType<typeof setTimeout> | null = null;
function onSynopsisInput(v: string) {
  if (!props.doc || props.readOnly) return;
  if (synopsisDebounce) clearTimeout(synopsisDebounce);
  // Debounce so we don't fire an IPC roundtrip on every keystroke.
  synopsisDebounce = setTimeout(() => {
    const trimmed = v.trim();
    emit('synopsisChange', trimmed.length === 0 ? null : trimmed);
  }, 400);
}
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
          <div class="flex justify-between items-center gap-2">
            <dt class="opacity-60">{{ t('status.label') }}</dt>
            <Select
              :model-value="doc.status"
              :options="statusOptions"
              option-label="label"
              option-value="value"
              :disabled="readOnly"
              class="!min-w-[8rem]"
              size="small"
              :pt="{ root: { class: '!text-xs' } }"
              @update:model-value="onStatusChange"
            />
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
        <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
          {{ t('synopsis.label') }}
        </h4>
        <Textarea
          :model-value="doc.synopsis ?? ''"
          :placeholder="t('synopsis.placeholder')"
          :disabled="readOnly"
          rows="3"
          auto-resize
          class="w-full !text-sm"
          @update:model-value="onSynopsisInput"
        />
      </section>

      <section>
        <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
          {{ t('goal.documentLabel') }}
        </h4>
        <GoalProgress
          :current="wordCountHere"
          :goal="doc.goalWords ?? null"
          :read-only="readOnly"
          @update:goal="(v) => emit('goalChange', v)"
        />
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

      <section>
        <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
          {{ t('tags.label') }}
        </h4>
        <Chips
          :model-value="doc.tags"
          :placeholder="t('tags.placeholder')"
          :disabled="readOnly"
          separator=","
          class="w-full"
          @update:model-value="onTagsChange"
        />
      </section>

      <SnapshotsList
        :document-id="doc.id"
        :current-html="doc.content ?? ''"
        :read-only="readOnly"
        @restored="emit('snapshotRestored')"
      />

      <section v-if="readOnly" class="text-xs opacity-70 italic">
        {{ t('project.readOnlyBanner') }}
      </section>
    </div>
  </aside>
</template>
