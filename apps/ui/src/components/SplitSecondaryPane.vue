<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Select from 'primevue/select';
import Button from 'primevue/button';
import TipTapEditor from '@/editor/TipTapEditor.vue';
import { useEditorAutoSave } from '@/composables/useEditorAutoSave';
import { useEditorSettings } from '@/composables/useEditorSettings';
import { useDocumentStore } from '@/stores/document';
import { useUiStore } from '@/stores/ui';
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
const uiStore = useUiStore();
const { autosaveMs } = useEditorSettings();

const saving = ref(false);
const editorDoc = useEditorAutoSave({
  persist: async (id, payload) => {
    saving.value = true;
    try {
      await docStore.save(id, payload);
    } finally {
      saving.value = false;
    }
  },
  readOnly: () => props.readOnly,
  delay: () => autosaveMs.value,
});
const editorContent = editorDoc.content;
const editorContentJson = editorDoc.contentJson;
/** When locked, the pane pins its current doc — the picker/bookmarks are
 *  disabled so it can't be swapped by accident (K-10). */
const locked = ref(false);

/** Recently-opened docs in this pane (excluding the current + primary). */
const bookmarks = computed(() =>
  uiStore
    .getSplitBookmarks(props.projectId)
    .filter((id) => id !== props.secondaryDocId && id !== props.primaryDocId)
    .map((id) => docStore.documents.find((d: DocNode) => d.id === id))
    .filter((d): d is DocNode => !!d && d.docType !== 'folder')
    .slice(0, 5),
);

function openBookmark(id: string) {
  if (locked.value) return;
  emit('update:secondaryDocId', id);
}

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
      await editorDoc.load(null);
      return;
    }
    if (editorDoc.boundId.value === id) return;
    try {
      const doc = await ipc.getDocument(id);
      if (!doc) return;
      // load() flushes the previously-loaded doc's pending edit before swapping
      // in the new content, so switching panes can't discard edits (AUD-02).
      await editorDoc.load({ id, content: doc.content, contentJson: doc.contentJson });
      uiStore.pushSplitBookmark(props.projectId, id);
    } catch {
      await editorDoc.load(null);
    }
  },
  { immediate: true },
);

function onInput(v: string) {
  editorDoc.onContent(v);
}
function onJsonInput(v: string) {
  editorDoc.onContentJson(v);
}
</script>

<template>
  <div class="flex flex-col h-full min-h-0 bg-surface-0 dark:bg-surface-950">
    <div
      class="h-10 px-3 flex items-center gap-2 border-b border-surface-200 dark:border-surface-700"
    >
      <Select
        v-if="!locked"
        v-model="secondaryModel"
        :options="docOptions"
        option-label="label"
        option-value="value"
        :placeholder="t('split.pickDocument')"
        class="flex-1"
        size="small"
      />
      <span v-else class="flex-1 min-w-0 truncate text-sm font-medium">
        {{ secondaryDoc?.title || t('project.untitled') }}
      </span>
      <span
        v-if="saving"
        class="text-[10px] uppercase tracking-wide opacity-60"
        :aria-label="t('split.saving')"
      >
        {{ t('split.saving') }}
      </span>
      <Button
        v-tooltip.bottom="t('split.lock')"
        :aria-label="t('split.lock')"
        :aria-pressed="locked"
        :icon="locked ? 'pi pi-lock' : 'pi pi-lock-open'"
        text
        size="small"
        severity="secondary"
        :class="{ 'p-button-outlined': locked }"
        @click="locked = !locked"
      />
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
    <div
      v-if="!locked && bookmarks.length > 0"
      class="flex items-center gap-1 flex-wrap px-3 py-1 border-b border-surface-200 dark:border-surface-700"
    >
      <button
        v-for="b in bookmarks"
        :key="b.id"
        class="rounded-full bg-surface-100 dark:bg-surface-800 px-2 py-0.5 text-[11px] max-w-[10rem] truncate hover:bg-surface-200 dark:hover:bg-surface-700"
        @click="openBookmark(b.id)"
      >
        {{ b.title || t('project.untitled') }}
      </button>
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
