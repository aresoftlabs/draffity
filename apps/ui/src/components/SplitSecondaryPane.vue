<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Select from 'primevue/select';
import Button from 'primevue/button';
import TipTapEditor from '@/editor/TipTapEditor.vue';
import { useAutoSave } from '@/composables/useAutoSave';
import { useDocumentStore } from '@/stores/document';
import { ipc } from '@/services/ipc';
import type { DocNode } from '@draffity/shared-types';

/**
 * Secondary pane for the split-editor view. Picks any document in the
 * current project (except the primary one), loads its content directly
 * from storage (so the primary pane's selection isn't touched), and
 * autosaves edits back to the same document. The primary pane keeps its
 * own state via the document store.
 */

const props = defineProps<{
  projectId: string;
  primaryDocId: string | null;
  secondaryDocId: string | null;
  readOnly: boolean;
}>();

const emit = defineEmits<{
  'update:secondaryDocId': [value: string | null];
  close: [];
}>();

const { t } = useI18n();
const docStore = useDocumentStore();

const editorContent = ref('');
const editorContentJson = ref<string | null>(null);
const loadedId = ref<string | null>(null);
const saving = ref(false);

const docOptions = computed<{ label: string; value: string }[]>(() =>
  docStore.documents
    .filter((d: DocNode) => d.docType !== 'folder' && d.id !== props.primaryDocId)
    .map((d: DocNode) => ({ label: d.title || t('project.untitled'), value: d.id })),
);

const secondaryDoc = computed<DocNode | null>(() => {
  if (!props.secondaryDocId) return null;
  return docStore.documents.find((d: DocNode) => d.id === props.secondaryDocId) ?? null;
});

const secondaryModel = computed({
  get: () => props.secondaryDocId,
  set: (v: string | null) => emit('update:secondaryDocId', v),
});

watch(
  () => props.secondaryDocId,
  async (id) => {
    if (!id) {
      editorContent.value = '';
      editorContentJson.value = null;
      loadedId.value = null;
      return;
    }
    if (loadedId.value === id) return;
    try {
      const doc = await ipc.getDocument(id);
      if (!doc) return;
      editorContent.value = doc.content ?? '';
      editorContentJson.value = doc.contentJson ?? null;
      loadedId.value = id;
    } catch {
      editorContent.value = '';
      editorContentJson.value = null;
    }
  },
  { immediate: true },
);

const auto = useAutoSave(async () => {
  const id = props.secondaryDocId;
  if (!id || props.readOnly) return;
  saving.value = true;
  try {
    await docStore.save(id, {
      content: editorContent.value,
      contentJson: editorContentJson.value ?? undefined,
    });
  } finally {
    saving.value = false;
  }
}, 500);

function onInput(v: string) {
  editorContent.value = v;
  auto.trigger();
}
function onJsonInput(v: string) {
  editorContentJson.value = v;
  auto.trigger();
}
</script>

<template>
  <div class="flex flex-col h-full min-h-0 bg-surface-0 dark:bg-surface-950">
    <div
      class="h-10 px-3 flex items-center gap-2 border-b border-surface-200 dark:border-surface-700"
    >
      <Select
        v-model="secondaryModel"
        :options="docOptions"
        option-label="label"
        option-value="value"
        :placeholder="t('split.pickDocument')"
        class="flex-1"
        size="small"
      />
      <span
        v-if="saving"
        class="text-[10px] uppercase tracking-wide opacity-60"
        :aria-label="t('split.saving')"
      >
        {{ t('split.saving') }}
      </span>
      <Button
        v-tooltip.bottom="t('split.close')"
        :aria-label="t('split.close')"
        icon="pi pi-times"
        text
        size="small"
        severity="secondary"
        @click="emit('close')"
      />
    </div>
    <div class="flex-1 min-h-0 overflow-auto">
      <div v-if="!secondaryDoc" class="h-full flex items-center justify-center text-sm opacity-60">
        {{ t('split.empty') }}
      </div>
      <TipTapEditor
        v-else
        :model-value="editorContent"
        :model-value-json="editorContentJson"
        :editable="!readOnly"
        :placeholder="t('project.untitled')"
        @update:model-value="onInput"
        @update:model-value-json="onJsonInput"
      />
    </div>
  </div>
</template>
