<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import SelectButton from 'primevue/selectbutton';
import MultiSelect from 'primevue/multiselect';
import type { DocumentStatus } from '@draffity/shared-types';
import { useIpcError } from '@/composables/useIpcError';
import { ipc, type Collection, type CollectionKind } from '@/services/ipc';

const props = defineProps<{
  visible: boolean;
  projectId: string;
  /** When set, the dialog edits this collection; otherwise it creates one. */
  collection?: Collection | null;
}>();
const emit = defineEmits<{ 'update:visible': [boolean]; saved: [] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();

const name = ref('');
const kind = ref<CollectionKind>('manual');
const tagsAny = ref<string[]>([]);
const statuses = ref<DocumentStatus[]>([]);
const titleContains = ref('');
const projectTags = ref<string[]>([]);
const saving = ref(false);

const isEdit = computed(() => !!props.collection);

const kindOptions = computed(() => [
  { label: t('collections.kindManual'), value: 'manual' as CollectionKind },
  { label: t('collections.kindSmart'), value: 'smart' as CollectionKind },
]);

const statusOptions = computed(() => [
  { label: t('collections.status.draft'), value: 'draft' as DocumentStatus },
  { label: t('collections.status.revised'), value: 'revised' as DocumentStatus },
  { label: t('collections.status.final'), value: 'final' as DocumentStatus },
  { label: t('collections.status.trashed'), value: 'trashed' as DocumentStatus },
]);

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

const queryHasFilter = computed(
  () => tagsAny.value.length > 0 || statuses.value.length > 0 || titleContains.value.trim() !== '',
);
const canSave = computed(
  () => name.value.trim() !== '' && (kind.value === 'manual' || queryHasFilter.value),
);

watch(
  () => props.visible,
  async (v) => {
    if (!v) return;
    const c = props.collection;
    name.value = c?.name ?? '';
    kind.value = c?.kind ?? 'manual';
    tagsAny.value = c?.query?.tagsAny ?? [];
    statuses.value = c?.query?.statuses ?? [];
    titleContains.value = c?.query?.titleContains ?? '';
    projectTags.value = (await ipcRun('', () => ipc.listProjectTags(props.projectId))) ?? [];
  },
);

function buildQuery() {
  return {
    tagsAny: tagsAny.value,
    statuses: statuses.value,
    titleContains: titleContains.value.trim() || null,
  };
}

async function onSave() {
  if (!canSave.value) return;
  saving.value = true;
  try {
    if (isEdit.value && props.collection) {
      const c = props.collection;
      if (name.value.trim() !== c.name) {
        await ipcRun(t('collections.error'), () => ipc.renameCollection(c.id, name.value.trim()));
      }
      if (c.kind === 'smart') {
        await ipcRun(t('collections.error'), () => ipc.setCollectionQuery(c.id, buildQuery()));
      }
    } else {
      await ipcRun(t('collections.error'), () =>
        ipc.createCollection({
          projectId: props.projectId,
          name: name.value.trim(),
          kind: kind.value,
          query: kind.value === 'smart' ? buildQuery() : null,
        }),
      );
    }
    emit('saved');
    visibleModel.value = false;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :header="isEdit ? t('collections.editTitle') : t('collections.newTitle')"
    :style="{ width: '28rem', maxWidth: '95vw' }"
  >
    <div class="space-y-3">
      <InputText v-model="name" class="w-full" :placeholder="t('collections.namePlaceholder')" />

      <SelectButton
        v-if="!isEdit"
        v-model="kind"
        :options="kindOptions"
        option-label="label"
        option-value="value"
        :allow-empty="false"
      />

      <template v-if="kind === 'smart'">
        <div>
          <label class="text-xs opacity-70 block mb-1">{{ t('collections.filterTags') }}</label>
          <MultiSelect
            v-model="tagsAny"
            :options="projectTags"
            class="w-full"
            display="chip"
            :placeholder="t('collections.anyTag')"
          />
        </div>
        <div>
          <label class="text-xs opacity-70 block mb-1">{{ t('collections.filterStatus') }}</label>
          <MultiSelect
            v-model="statuses"
            :options="statusOptions"
            option-label="label"
            option-value="value"
            class="w-full"
            :placeholder="t('collections.anyStatus')"
          />
        </div>
        <div>
          <label class="text-xs opacity-70 block mb-1">{{ t('collections.filterTitle') }}</label>
          <InputText
            v-model="titleContains"
            class="w-full"
            :placeholder="t('collections.titlePlaceholder')"
          />
        </div>
        <p v-if="!queryHasFilter" class="text-xs text-amber-600 dark:text-amber-400">
          {{ t('collections.needFilter') }}
        </p>
      </template>
    </div>

    <template #footer>
      <Button
        :label="t('actions.cancel')"
        text
        severity="secondary"
        @click="visibleModel = false"
      />
      <Button :label="t('actions.save')" :loading="saving" :disabled="!canSave" @click="onSave" />
    </template>
  </Dialog>
</template>
