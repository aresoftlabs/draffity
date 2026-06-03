<script setup lang="ts">
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import DatePicker from 'primevue/datepicker';
import Select from 'primevue/select';
import { useI18n } from 'vue-i18n';
import type { CustomField } from '@draffity/shared-types';

const props = defineProps<{
  fields: CustomField[];
  /** Current values keyed by field id (from `DocNode.metadata`). */
  values: Record<string, string>;
  readOnly?: boolean;
}>();

const emit = defineEmits<{ change: [fieldId: string, value: string | null] }>();

const { t } = useI18n();

function commit(fieldId: string, raw: string | null) {
  if (props.readOnly) return;
  const value = raw == null || raw.trim() === '' ? null : raw.trim();
  emit('change', fieldId, value);
}

/** `yyyy-mm-dd` string â†’ Date for the picker model (local midnight). */
function toDate(v: string | undefined): Date | null {
  if (!v) return null;
  const d = new Date(`${v}T00:00:00`);
  return Number.isNaN(d.getTime()) ? null : d;
}

/** Date â†’ `yyyy-mm-dd` for storage (locale-independent). */
function fromDate(d: Date | null): string | null {
  if (!d) return null;
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function toNum(v: string | undefined): number | null {
  if (v == null || v === '') return null;
  const n = Number(v);
  return Number.isNaN(n) ? null : n;
}
</script>

<template>
  <div class="space-y-3">
    <div v-for="f in fields" :key="f.id">
      <label class="text-xs opacity-70 block mb-1">{{ f.name }}</label>

      <InputText
        v-if="f.kind === 'text'"
        :model-value="values[f.id] ?? ''"
        :disabled="readOnly"
        class="w-full !text-sm"
        @blur="(e: FocusEvent) => commit(f.id, (e.target as HTMLInputElement).value)"
      />

      <InputNumber
        v-else-if="f.kind === 'number'"
        :model-value="toNum(values[f.id])"
        :disabled="readOnly"
        class="w-full"
        size="small"
        @update:model-value="(v: number | null) => commit(f.id, v == null ? null : String(v))"
      />

      <DatePicker
        v-else-if="f.kind === 'date'"
        :model-value="toDate(values[f.id])"
        :disabled="readOnly"
        show-icon
        show-button-bar
        date-format="yy-mm-dd"
        class="w-full"
        size="small"
        @update:model-value="(v) => commit(f.id, fromDate(v as Date | null))"
      />

      <Select
        v-else-if="f.kind === 'select'"
        :model-value="values[f.id] ?? null"
        :options="f.options"
        :disabled="readOnly"
        show-clear
        :placeholder="t('customFields.selectPlaceholder')"
        class="w-full !text-sm"
        size="small"
        @update:model-value="(v: string | null) => commit(f.id, v)"
      />
    </div>
  </div>
</template>
