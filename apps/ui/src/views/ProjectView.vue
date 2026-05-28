<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import type { DocumentStatus, DocumentType } from '@draffity/shared-types';

import { useProjectStore } from '@/stores/project';
import { useDocumentStore, type ReorderOp } from '@/stores/document';
import { useUiStore } from '@/stores/ui';
import { useIpcError } from '@/composables/useIpcError';
import { useAutoSave } from '@/composables/useAutoSave';
import { useShortcuts } from '@/composables/useShortcuts';

import Binder from '@/components/Binder.vue';
import Inspector from '@/components/Inspector.vue';
import SaveIndicator from '@/components/SaveIndicator.vue';
import ExportDialog from '@/components/ExportDialog.vue';
import SearchDialog from '@/components/SearchDialog.vue';
import FindReplaceBar from '@/components/FindReplaceBar.vue';
import GoalProgress from '@/components/GoalProgress.vue';
import TipTapEditor from '@/editor/TipTapEditor.vue';
import EditorToolbar from '@/editor/EditorToolbar.vue';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const docStore = useDocumentStore();
const uiStore = useUiStore();
const { run } = useIpcError();

const focusMode = computed(() => uiStore.focusMode);

function toggleFocus() {
  uiStore.toggleFocusMode();
}

const projectId = computed(() => String(route.params.id));
const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value) ?? null);
const readOnly = computed(() => project.value?.status === 'archived');

const { selected, saveState, lastSavedAt, wordCount, totalWordCount } = storeToRefs(docStore);

const editorContent = ref('');
const editorRef = ref<InstanceType<typeof TipTapEditor> | null>(null);
const editor = computed(() => editorRef.value?.editor ?? null);

const showExport = ref(false);
const showSearch = ref(false);
const findVisible = ref(false);
const findMode = ref<'find' | 'replace'>('find');
const sessionStartWords = ref(0);
const sessionWordCount = computed(() =>
  Math.max(0, totalWordCount.value - sessionStartWords.value),
);

const auto = useAutoSave(async () => {
  if (!selected.value) return;
  if (readOnly.value) return;
  await run(t('errors.saveDocument'), () =>
    docStore.save(selected.value!.id, { content: editorContent.value }),
  );
}, 500);

async function loadProject() {
  if (!projectStore.projects.length) {
    await run(t('errors.loadProjects'), () => projectStore.loadAll());
  }
  if (!project.value) {
    router.replace({ name: 'dashboard' });
    return;
  }
  await run(t('errors.loadDocuments'), () => docStore.loadFor(projectId.value));
  // Snapshot the word count at load time so the inspector can show
  // "words written this session" without a roundtrip.
  sessionStartWords.value = totalWordCount.value;
  syncEditorFromSelection();
}

async function reloadAfterRestore() {
  await run(t('errors.loadDocuments'), () => docStore.loadFor(projectId.value));
  syncEditorFromSelection();
}

function onSnapshotRestored() {
  void reloadAfterRestore();
}

function syncEditorFromSelection() {
  editorContent.value = selected.value?.content ?? '';
}

watch(selected, () => {
  // When the selection changes, flush pending save then load new content.
  void auto.flush().then(syncEditorFromSelection);
});

watch(projectId, async (next, prev) => {
  if (next !== prev) {
    docStore.reset();
    await loadProject();
  }
});

function onEditorInput(value: string) {
  editorContent.value = value;
  if (!readOnly.value) auto.trigger();
}

async function onCreate(type: DocumentType) {
  if (!project.value) return;
  const titleByType: Record<DocumentType, string> = {
    chapter: t('project.newChapter'),
    scene: t('project.newScene'),
    note: t('project.newNote'),
    folder: t('project.newDocument'),
    manga_page: t('project.newDocument'),
  };
  await run(t('errors.createDocument'), () =>
    docStore.create({
      projectId: project.value!.id,
      title: titleByType[type],
      docType: type,
      content: '',
    }),
  );
}

function onSelect(id: string) {
  docStore.select(id);
}

async function onReorder(ops: ReorderOp[]) {
  if (!project.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.reorder(project.value!.id, ops));
}

async function onStatusChange(status: DocumentStatus) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setStatus(selected.value!.id, status));
}

async function onTagsChange(tags: string[]) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setTags(selected.value!.id, tags));
}

async function onDocGoalChange(goal: number | null) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setGoal(selected.value!.id, goal));
}

async function onProjectGoalChange(goal: number | null) {
  if (!project.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => projectStore.setGoal(project.value!.id, goal));
}

useShortcuts({
  'ctrl+s': () => {
    void auto.flush();
  },
  'ctrl+n': () => {
    if (!readOnly.value) onCreate('chapter');
  },
  'ctrl+shift+f': () => {
    showSearch.value = true;
  },
  'ctrl+f': () => {
    findMode.value = 'find';
    findVisible.value = true;
  },
  'ctrl+h': () => {
    if (readOnly.value) return;
    findMode.value = 'replace';
    findVisible.value = true;
  },
  f11: () => toggleFocus(),
});

function onSearchJump(documentId: string) {
  docStore.select(documentId);
}

onMounted(loadProject);
</script>

<template>
  <div v-if="project" class="flex-1 flex flex-col min-h-0">
    <header
      class="h-10 px-4 flex items-center gap-3 border-b border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-950"
    >
      <Button
        icon="pi pi-arrow-left"
        text
        severity="secondary"
        size="small"
        :aria-label="t('actions.back')"
        @click="router.push('/')"
      />
      <h2 class="text-sm font-semibold truncate">{{ project.title }}</h2>
      <Tag v-if="readOnly" :value="t('dashboard.readOnly')" severity="secondary" class="ml-1" />
      <div class="flex items-center gap-2 min-w-[12rem] max-w-[20rem]">
        <GoalProgress
          :current="totalWordCount"
          :goal="project.goalWords ?? null"
          :read-only="readOnly"
          compact
          @update:goal="onProjectGoalChange"
        />
      </div>
      <span class="flex-1" />
      <SaveIndicator :state="saveState" :last-saved-at="lastSavedAt" />
      <Button
        v-tooltip.bottom="t('search.button')"
        icon="pi pi-search"
        text
        severity="secondary"
        size="small"
        :aria-label="t('search.button')"
        @click="showSearch = true"
      />
      <Button
        v-tooltip.bottom="t('project.focusMode')"
        :icon="focusMode ? 'pi pi-window-minimize' : 'pi pi-arrows-alt'"
        text
        severity="secondary"
        size="small"
        :aria-label="t('project.focusMode')"
        :aria-pressed="focusMode"
        @click="toggleFocus"
      />
      <Button
        icon="pi pi-download"
        text
        severity="secondary"
        size="small"
        :aria-label="t('actions.export')"
        @click="showExport = true"
      />
    </header>

    <ExportDialog v-model:visible="showExport" :project="project" />
    <SearchDialog v-model:visible="showSearch" :project-id="project.id" @jump="onSearchJump" />

    <Splitter
      class="flex-1 !rounded-none !border-0 min-h-0"
      :pt="{
        gutter: { class: 'bg-surface-200 dark:bg-surface-700' },
      }"
      style-class="h-full"
    >
      <SplitterPanel v-if="!focusMode" :size="22" :min-size="14" class="!min-w-0">
        <Binder
          :documents="docStore.documents"
          :selected-id="docStore.selectedId"
          :read-only="readOnly"
          @select="onSelect"
          @create="onCreate"
          @reorder="onReorder"
        />
      </SplitterPanel>

      <SplitterPanel :size="focusMode ? 100 : 56" :min-size="30" class="!min-w-0 flex flex-col">
        <div
          v-if="readOnly"
          class="px-4 py-2 text-xs italic bg-amber-100 dark:bg-amber-900/30 text-amber-900 dark:text-amber-200 border-b border-amber-300 dark:border-amber-800"
        >
          {{ t('project.readOnlyBanner') }}
        </div>
        <EditorToolbar :editor="editor" :disabled="readOnly" />
        <FindReplaceBar
          v-model:visible="findVisible"
          :editor="editor"
          :mode="findMode"
          :read-only="readOnly"
        />
        <div class="flex-1 min-h-0 bg-surface-0 dark:bg-surface-950">
          <div v-if="!selected" class="h-full flex items-center justify-center text-sm opacity-60">
            {{ t('project.noSelection') }}
          </div>
          <TipTapEditor
            v-else
            ref="editorRef"
            :model-value="editorContent"
            :editable="!readOnly"
            :placeholder="t('project.untitled')"
            @update:model-value="onEditorInput"
          />
        </div>
      </SplitterPanel>

      <SplitterPanel v-if="!focusMode" :size="22" :min-size="14" class="!min-w-0">
        <Inspector
          :doc="selected"
          :word-count-here="wordCount"
          :word-count-total="totalWordCount"
          :session-word-count="sessionWordCount"
          :read-only="readOnly"
          @snapshot-restored="onSnapshotRestored"
          @status-change="onStatusChange"
          @tags-change="onTagsChange"
          @goal-change="onDocGoalChange"
        />
      </SplitterPanel>
    </Splitter>
  </div>
</template>
