<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Select from 'primevue/select';
import Chips from 'primevue/chips';
import MultiSelect from 'primevue/multiselect';
import Button from 'primevue/button';
import ToggleSwitch from 'primevue/toggleswitch';
import SelectButton from 'primevue/selectbutton';
import Textarea from 'primevue/textarea';
import type { CustomField, DocNode, DocumentStatus, Label } from '@draffity/shared-types';
import SnapshotsList from '@/components/SnapshotsList.vue';
import GoalProgress from '@/components/GoalProgress.vue';
import CustomFieldsEditor from '@/components/CustomFieldsEditor.vue';

const props = defineProps<{
  doc: DocNode | null;
  wordCountHere: number;
  wordCountTotal: number;
  sessionWordCount?: number;
  /** Project labels available to assign (I-05/I-06). */
  labels?: Label[];
  /** Project custom metadata fields (I-08/I-09). */
  customFields?: CustomField[];
  /** Reading speed (words/minute) for reading-time estimates (J-09). */
  readingWpm?: number;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  snapshotRestored: [];
  statusChange: [status: DocumentStatus];
  tagsChange: [tags: string[]];
  labelsChange: [labelIds: string[]];
  manageLabels: [];
  metadataChange: [fieldId: string, value: string | null];
  manageFields: [];
  researchChange: [isResearch: boolean];
  matterChange: [isFront: boolean, isBack: boolean];
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

/** Reading-time estimate (J-09): words / wpm, floored to a friendly label. */
function readingTime(words: number): string {
  const wpm = props.readingWpm && props.readingWpm > 0 ? props.readingWpm : 200;
  const minutes = words / wpm;
  if (minutes < 1) return t('reading.lessThanMinute');
  return t('reading.minutes', { n: Math.round(minutes) });
}

type MatterValue = 'body' | 'front' | 'back';
const matterValue = computed<MatterValue>(() =>
  props.doc?.isFrontMatter ? 'front' : props.doc?.isBackMatter ? 'back' : 'body',
);
const matterOptions = computed(() => [
  { value: 'body' as MatterValue, label: t('compile.matterBody') },
  { value: 'front' as MatterValue, label: t('compile.matterFront') },
  { value: 'back' as MatterValue, label: t('compile.matterBack') },
]);
function onMatterChange(v: MatterValue) {
  if (!props.doc || props.readOnly || !v) return;
  emit('matterChange', v === 'front', v === 'back');
}

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

function onLabelsChange(next: string[]) {
  if (!props.doc || props.readOnly) return;
  emit('labelsChange', next);
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
          <div class="flex justify-between items-center gap-2 pt-1">
            <dt class="opacity-60">{{ t('research.label') }}</dt>
            <ToggleSwitch
              :model-value="doc.isResearch"
              :disabled="readOnly"
              :aria-label="t('research.toggle')"
              @update:model-value="(v: boolean) => emit('researchChange', v)"
            />
          </div>
        </dl>
        <p class="text-[11px] opacity-50 mt-1">{{ t('research.hint') }}</p>
        <div class="flex items-center justify-between gap-2 mt-2">
          <span class="text-xs opacity-60">{{ t('compile.matter') }}</span>
          <SelectButton
            :model-value="matterValue"
            :options="matterOptions"
            option-label="label"
            option-value="value"
            :allow-empty="false"
            :disabled="readOnly"
            size="small"
            @update:model-value="onMatterChange"
          />
        </div>
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
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('reading.label') }}</dt>
            <dd>{{ readingTime(wordCountTotal) }}</dd>
          </div>
        </dl>
      </section>

      <section>
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60">
            {{ t('labels.label') }}
          </h4>
          <Button
            icon="pi pi-cog"
            text
            size="small"
            :pt="{ root: { class: '!w-6 !h-6 !p-0' } }"
            :aria-label="t('labels.manageTitle')"
            @click="emit('manageLabels')"
          />
        </div>
        <MultiSelect
          :model-value="doc.labelIds"
          :options="labels ?? []"
          option-label="name"
          option-value="id"
          :disabled="readOnly"
          display="chip"
          filter
          :placeholder="t('labels.assignPlaceholder')"
          :empty-message="t('labels.empty')"
          class="w-full !text-sm"
          @update:model-value="onLabelsChange"
        >
          <template #option="{ option }">
            <span class="flex items-center gap-2">
              <span
                class="w-2.5 h-2.5 rounded-full shrink-0"
                :style="{ backgroundColor: option.color }"
                aria-hidden="true"
              />
              <span class="truncate">{{ option.name }}</span>
            </span>
          </template>
        </MultiSelect>
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

      <section>
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60">
            {{ t('customFields.label') }}
          </h4>
          <Button
            icon="pi pi-cog"
            text
            size="small"
            :pt="{ root: { class: '!w-6 !h-6 !p-0' } }"
            :aria-label="t('customFields.manageTitle')"
            @click="emit('manageFields')"
          />
        </div>
        <CustomFieldsEditor
          v-if="customFields && customFields.length > 0"
          :fields="customFields"
          :values="doc.metadata"
          :read-only="readOnly"
          @change="(fieldId, value) => emit('metadataChange', fieldId, value)"
        />
        <p v-else class="text-xs opacity-50">{{ t('customFields.empty') }}</p>
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
