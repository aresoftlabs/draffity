<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import type { Label } from '@draffity/shared-types';
import { useIpcError } from '@/composables/useIpcError';
import { useLabelStore } from '@/stores/labels';

const props = defineProps<{
  visible: boolean;
  projectId: string;
}>();
const emit = defineEmits<{ 'update:visible': [boolean] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();
const labelStore = useLabelStore();

/** Curated palette â€” a fixed set keeps labels visually coherent and sidesteps
 *  free-form color-picker contrast pitfalls. */
const PALETTE = [
  '#ef4444',
  '#f97316',
  '#f59e0b',
  '#22c55e',
  '#14b8a6',
  '#3b82f6',
  '#6366f1',
  '#8b5cf6',
  '#ec4899',
  '#6b7280',
];

const name = ref('');
const color = ref<string>(PALETTE[0]);
const editingId = ref<string | null>(null);
const saving = ref(false);

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

const isEditing = computed(() => editingId.value !== null);
const canSave = computed(() => name.value.trim() !== '');

function resetForm() {
  name.value = '';
  color.value = PALETTE[0];
  editingId.value = null;
}

function startEdit(label: Label) {
  editingId.value = label.id;
  name.value = label.name;
  color.value = label.color;
}

async function onSave() {
  if (!canSave.value) return;
  saving.value = true;
  try {
    if (editingId.value) {
      await ipcRun(t('labels.error'), () =>
        labelStore.update(editingId.value!, name.value.trim(), color.value),
      );
    } else {
      await ipcRun(t('labels.error'), () =>
        labelStore.create({
          projectId: props.projectId,
          name: name.value.trim(),
          color: color.value,
        }),
      );
    }
    resetForm();
  } finally {
    saving.value = false;
  }
}

async function onDelete(label: Label) {
  if (!confirm(t('labels.confirmDelete', { name: label.name }))) return;
  await ipcRun(t('labels.error'), () => labelStore.remove(label.id));
  if (editingId.value === label.id) resetForm();
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
    :header="t('labels.manageTitle')"
    :style="{ width: '30rem', maxWidth: '95vw' }"
  >
    <div class="space-y-4">
      <!-- Existing labels -->
      <ul v-if="labelStore.labels.length > 0" class="space-y-1">
        <li
          v-for="label in labelStore.labels"
          :key="label.id"
          class="group flex items-center gap-2 px-2 py-1 rounded hover:bg-surface-100 dark:hover:bg-surface-800"
        >
          <span
            class="w-3 h-3 rounded-full shrink-0"
            :style="{ backgroundColor: label.color }"
            aria-hidden="true"
          />
          <span class="flex-1 min-w-0 truncate text-sm">{{ label.name }}</span>
          <Button
            icon="pi pi-pencil"
            text
            size="small"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('labels.edit')"
            @click="startEdit(label)"
          />
          <Button
            icon="pi pi-trash"
            text
            size="small"
            severity="danger"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('labels.delete')"
            @click="onDelete(label)"
          />
        </li>
      </ul>
      <p v-else class="text-sm opacity-50">{{ t('labels.empty') }}</p>

      <!-- Create / edit form -->
      <div class="border-t border-surface-200 dark:border-surface-700 pt-3 space-y-2">
        <label class="text-xs opacity-70 block">
          {{ isEditing ? t('labels.editTitle') : t('labels.newTitle') }}
        </label>
        <InputText
          v-model="name"
          class="w-full"
          :placeholder="t('labels.namePlaceholder')"
          @keydown.enter="onSave"
        />
        <div class="flex flex-wrap gap-2" role="radiogroup" :aria-label="t('labels.color')">
          <button
            v-for="c in PALETTE"
            :key="c"
            type="button"
            class="w-6 h-6 rounded-full border-2 transition-transform"
            :class="
              color === c
                ? 'border-surface-900 dark:border-surface-0 scale-110'
                : 'border-transparent'
            "
            :style="{ backgroundColor: c }"
            :aria-label="c"
            :aria-pressed="color === c"
            @click="color = c"
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
            :label="isEditing ? t('actions.save') : t('labels.add')"
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
