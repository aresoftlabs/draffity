<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Textarea from 'primevue/textarea';
import DatePicker from 'primevue/datepicker';
import Message from 'primevue/message';
import type { Template } from '@draffity/shared-types';

const props = defineProps<{
  template: Template | null;
  title: string;
  values: Record<string, unknown>;
}>();

const emit = defineEmits<{
  'update:title': [v: string];
  'update:values': [v: Record<string, unknown>];
}>();

const { t } = useI18n();

const requiredMissing = computed(() => {
  if (!props.template) return false;
  return props.template.metadataFields.some((f) => {
    if (!f.required) return false;
    const v = props.values[f.key];
    if (v === undefined || v === null) return true;
    if (typeof v === 'string' && v.trim() === '') return true;
    return false;
  });
});

function updateField(key: string, v: unknown) {
  emit('update:values', { ...props.values, [key]: v });
}
</script>

<template>
  <div class="flex flex-col gap-4 min-h-[20rem]">
    <div class="flex flex-col gap-1">
      <label for="np-title" class="text-sm font-medium">
        {{ t('newProject.name') }}
        <span class="text-red-500">*</span>
      </label>
      <InputText
        id="np-title"
        :model-value="title"
        :placeholder="t('newProject.namePlaceholder')"
        autofocus
        @update:model-value="(v) => emit('update:title', v ?? '')"
      />
    </div>

    <div v-for="f in template?.metadataFields ?? []" :key="f.key" class="flex flex-col gap-1">
      <label :for="`np-meta-${f.key}`" class="text-sm font-medium">
        {{ f.label }}
        <span v-if="f.required" class="text-red-500">*</span>
      </label>
      <InputText
        v-if="f.type === 'string'"
        :id="`np-meta-${f.key}`"
        :model-value="(values[f.key] as string) ?? ''"
        @update:model-value="(v) => updateField(f.key, v)"
      />
      <Textarea
        v-else-if="f.type === 'text'"
        :id="`np-meta-${f.key}`"
        :model-value="(values[f.key] as string) ?? ''"
        rows="3"
        auto-resize
        @update:model-value="(v) => updateField(f.key, v)"
      />
      <InputNumber
        v-else-if="f.type === 'number'"
        :id="`np-meta-${f.key}`"
        :model-value="(values[f.key] as number) ?? null"
        @update:model-value="(v) => updateField(f.key, v)"
      />
      <DatePicker
        v-else-if="f.type === 'date'"
        :id="`np-meta-${f.key}`"
        :model-value="(values[f.key] as Date) ?? null"
        show-icon
        @update:model-value="(v) => updateField(f.key, v)"
      />
    </div>

    <Message v-if="requiredMissing" severity="warn" :closable="false">
      {{ t('newProject.fillRequired') }}
    </Message>
  </div>
</template>
