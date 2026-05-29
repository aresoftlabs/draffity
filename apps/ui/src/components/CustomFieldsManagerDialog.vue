<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import Chips from 'primevue/chips';
import type { CustomField, CustomFieldKind } from '@draffity/shared-types';
import { useIpcError } from '@/composables/useIpcError';
import { useCustomFieldStore } from '@/stores/customFields';

const props = defineProps<{
  visible: boolean;
  projectId: string;
}>();
const emit = defineEmits<{ 'update:visible': [boolean] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();
const fieldStore = useCustomFieldStore();

const name = ref('');
const kind = ref<CustomFieldKind>('text');
const options = ref<string[]>([]);
const editingId = ref<string | null>(null);
const saving = ref(false);

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

const kindOptions = computed<{ value: CustomFieldKind; label: string }[]>(() => [
  { value: 'text', label: t('customFields.kind.text') },
  { value: 'number', label: t('customFields.kind.number') },
  { value: 'date', label: t('customFields.kind.date') },
  { value: 'select', label: t('customFields.kind.select') },
]);

const isEditing = computed(() => editingId.value !== null);
const canSave = computed(
  () => name.value.trim() !== '' && (kind.value !== 'select' || options.value.length > 0),
);

function kindLabel(k: CustomFieldKind): string {
  return t(`customFields.kind.${k}`);
}

function resetForm() {
  name.value = '';
  kind.value = 'text';
  options.value = [];
  editingId.value = null;
}

function startEdit(field: CustomField) {
  editingId.value = field.id;
  name.value = field.name;
  kind.value = field.kind;
  options.value = [...field.options];
}

async function onSave() {
  if (!canSave.value) return;
  saving.value = true;
  try {
    if (editingId.value) {
      await ipcRun(t('customFields.error'), () =>
        fieldStore.update(editingId.value!, name.value.trim(), options.value),
      );
    } else {
      await ipcRun(t('customFields.error'), () =>
        fieldStore.create({
          projectId: props.projectId,
          name: name.value.trim(),
          kind: kind.value,
          options: kind.value === 'select' ? options.value : [],
        }),
      );
    }
    resetForm();
  } finally {
    saving.value = false;
  }
}

async function onDelete(field: CustomField) {
  if (!confirm(t('customFields.confirmDelete', { name: field.name }))) return;
  await ipcRun(t('customFields.error'), () => fieldStore.remove(field.id));
  if (editingId.value === field.id) resetForm();
}

watch(
  () => props.visible,
  (v) => {
    if (v) resetForm();
  },
);
</script>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :header="t('customFields.manageTitle')"
    :style="{ width: '32rem', maxWidth: '95vw' }"
  >
    <div class="space-y-4">
      <ul v-if="fieldStore.fields.length > 0" class="space-y-1">
        <li
          v-for="field in fieldStore.fields"
          :key="field.id"
          class="group flex items-center gap-2 px-2 py-1 rounded hover:bg-surface-100 dark:hover:bg-surface-800"
        >
          <span class="flex-1 min-w-0 truncate text-sm">{{ field.name }}</span>
          <span class="text-[10px] uppercase tracking-wide opacity-50">
            {{ kindLabel(field.kind) }}
          </span>
          <Button
            icon="pi pi-pencil"
            text
            size="small"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('customFields.edit')"
            @click="startEdit(field)"
          />
          <Button
            icon="pi pi-trash"
            text
            size="small"
            severity="danger"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('customFields.delete')"
            @click="onDelete(field)"
          />
        </li>
      </ul>
      <p v-else class="text-sm opacity-50">{{ t('customFields.empty') }}</p>

      <div class="border-t border-surface-200 dark:border-surface-700 pt-3 space-y-2">
        <label class="text-xs opacity-70 block">
          {{ isEditing ? t('customFields.editTitle') : t('customFields.newTitle') }}
        </label>
        <InputText v-model="name" class="w-full" :placeholder="t('customFields.namePlaceholder')" />
        <Select
          v-model="kind"
          :options="kindOptions"
          option-label="label"
          option-value="value"
          :disabled="isEditing"
          class="w-full"
          size="small"
        />
        <div v-if="kind === 'select'">
          <label class="text-xs opacity-70 block mb-1">{{ t('customFields.options') }}</label>
          <Chips
            v-model="options"
            :placeholder="t('customFields.optionsPlaceholder')"
            separator=","
            class="w-full"
          />
        </div>
        <div class="flex justify-end gap-2 pt-1">
          <Button
            v-if="isEditing"
            :label="t('actions.cancel')"
            text
            severity="secondary"
            size="small"
            @click="resetForm"
          />
          <Button
            :label="isEditing ? t('actions.save') : t('customFields.add')"
            icon="pi pi-check"
            size="small"
            :loading="saving"
            :disabled="!canSave"
            @click="onSave"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <Button :label="t('actions.close')" text @click="visibleModel = false" />
    </template>
  </Dialog>
</template>
