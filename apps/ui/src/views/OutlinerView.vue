<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import DataTable, { type DataTableCellEditCompleteEvent } from 'primevue/datatable';
import Column from 'primevue/column';
import InputText from 'primevue/inputtext';
import Textarea from 'primevue/textarea';
import Select from 'primevue/select';
import Tag from 'primevue/tag';
import type { DocNode, DocumentStatus } from '@draffity/shared-types';
import { countWords } from '@/stores/document';
import LabelChips from '@/components/LabelChips.vue';

const props = defineProps<{
  documents: DocNode[];
  selectedId: string | null;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  updateTitle: [payload: { id: string; title: string }];
  updateSynopsis: [payload: { id: string; synopsis: string | null }];
  updateStatus: [payload: { id: string; status: DocumentStatus }];
}>();

const { t } = useI18n();

// Same ordering as binder / corkboard so the user sees a consistent
// document list across views.
const rows = computed(() =>
  [...props.documents].sort((a, b) => {
    const pa = a.parentId ?? '';
    const pb = b.parentId ?? '';
    if (pa !== pb) return pa.localeCompare(pb);
    return a.position - b.position;
  }),
);

const statusOptions = computed(() =>
  (['draft', 'revised', 'final', 'trashed'] as const).map((value) => ({
    value,
    label: t(`status.${value}`),
  })),
);

function wordsOf(d: DocNode): number {
  return countWords(d.content ?? '');
}

function progressOf(d: DocNode): number | null {
  const g = d.goalWords ?? null;
  if (!g || g <= 0) return null;
  return Math.min(100, Math.round((wordsOf(d) / g) * 100));
}

function statusDotClass(s: DocumentStatus): string {
  switch (s) {
    case 'revised':
      return 'bg-blue-400';
    case 'final':
      return 'bg-emerald-500';
    case 'trashed':
      return 'bg-rose-400 opacity-60';
    default:
      return 'bg-surface-300 dark:bg-surface-600';
  }
}

/** Row highlight via the documented `row-class` prop. The previous `pt`
 *  approach reached `ctx.rowData.id` directly but PrimeVue 4 nests it
 *  under `ctx.context.rowData`, so the access threw on every cell during
 *  render and could freeze the panel on larger document trees. */
function rowClass(d: DocNode): string {
  return d.id === props.selectedId ? 'bg-primary-50 dark:bg-primary-900/20' : '';
}

function onCellEditComplete(event: DataTableCellEditCompleteEvent) {
  if (props.readOnly) return;
  const doc = event.data as DocNode;
  const field = event.field as keyof DocNode;
  if (field === 'title') {
    const v = String(event.newValue ?? '').trim();
    if (v.length === 0 || v === doc.title) return;
    emit('updateTitle', { id: doc.id, title: v });
  } else if (field === 'synopsis') {
    const raw = String(event.newValue ?? '').trim();
    const next = raw.length === 0 ? null : raw;
    if (next === (doc.synopsis ?? null)) return;
    emit('updateSynopsis', { id: doc.id, synopsis: next });
  }
}

function onStatusChange(d: DocNode, status: DocumentStatus) {
  if (props.readOnly || status === d.status) return;
  emit('updateStatus', { id: d.id, status });
}

function onRowClick(event: { data: DocNode }) {
  emit('select', event.data.id);
}
</script>

<template>
  <div class="h-full overflow-auto p-4 bg-surface-50/40 dark:bg-surface-900/40">
    <DataTable
      :value="rows"
      :edit-mode="readOnly ? undefined : 'cell'"
      :row-class="rowClass"
      row-hover
      class="text-sm"
      @cell-edit-complete="onCellEditComplete"
      @row-click="onRowClick"
    >
      <Column field="title" :header="t('outliner.title')" :style="{ minWidth: '14rem' }">
        <template #body="{ data }">
          <div class="flex items-center gap-2">
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="statusDotClass(data.status)" />
            <span class="font-medium truncate">{{ data.title || t('project.untitled') }}</span>
          </div>
        </template>
        <template #editor="{ data, field }">
          <InputText v-model="data[field]" autofocus class="w-full" />
        </template>
      </Column>

      <Column field="synopsis" :header="t('outliner.synopsis')" :style="{ minWidth: '20rem' }">
        <template #body="{ data }">
          <span v-if="data.synopsis" class="opacity-80 line-clamp-2">{{ data.synopsis }}</span>
          <span v-else class="italic opacity-40">{{ t('outliner.noSynopsis') }}</span>
        </template>
        <template #editor="{ data, field }">
          <Textarea v-model="data[field]" rows="2" auto-resize class="w-full" />
        </template>
      </Column>

      <Column
        field="words"
        :header="t('outliner.words')"
        :style="{ width: '9rem' }"
        body-class="!font-mono !text-xs"
      >
        <template #body="{ data }">
          <span>{{ wordsOf(data) }}</span>
          <span v-if="data.goalWords" class="opacity-60">/{{ data.goalWords }}</span>
          <span v-if="progressOf(data) !== null" class="ml-1 opacity-60">
            ({{ progressOf(data) }}%)
          </span>
        </template>
      </Column>

      <Column field="status" :header="t('status.label')" :style="{ width: '8rem' }">
        <template #body="{ data }">
          <Select
            :model-value="data.status"
            :options="statusOptions"
            option-label="label"
            option-value="value"
            :disabled="readOnly"
            size="small"
            class="!text-xs w-full"
            @update:model-value="(v: DocumentStatus) => onStatusChange(data, v)"
            @click.stop
          />
        </template>
      </Column>

      <Column field="labels" :header="t('labels.label')" :style="{ minWidth: '9rem' }">
        <template #body="{ data }">
          <LabelChips v-if="data.labelIds.length > 0" :label-ids="data.labelIds" />
          <span v-else class="italic opacity-40 text-xs">—</span>
        </template>
      </Column>

      <Column field="tags" :header="t('tags.label')" :style="{ minWidth: '10rem' }">
        <template #body="{ data }">
          <div v-if="data.tags.length > 0" class="flex gap-1 flex-wrap">
            <Tag
              v-for="tag in data.tags"
              :key="tag"
              :value="tag"
              severity="secondary"
              class="!text-[10px] !py-0 !px-1.5"
            />
          </div>
          <span v-else class="italic opacity-40 text-xs">—</span>
        </template>
      </Column>
    </DataTable>
  </div>
</template>
