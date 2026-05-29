<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { DocumentStatus, DocumentType } from '@draffity/shared-types';

import { useProjectStore } from '@/stores/project';
import { useDocumentStore, type ReorderOp } from '@/stores/document';
import { useUiStore, type ProjectViewMode } from '@/stores/ui';
import { useIpcError } from '@/composables/useIpcError';
import { useAutoSave } from '@/composables/useAutoSave';
import { useShortcuts } from '@/composables/useShortcuts';
import { useTypewriterScroll } from '@/composables/useTypewriterScroll';

import Binder from '@/components/Binder.vue';
import Inspector from '@/components/Inspector.vue';
import SaveIndicator from '@/components/SaveIndicator.vue';
import ExportDialog from '@/components/ExportDialog.vue';
import BibliographyDialog from '@/components/BibliographyDialog.vue';
import CitationPickerDialog from '@/components/CitationPickerDialog.vue';
import CodexRefPickerDialog from '@/components/CodexRefPickerDialog.vue';
import SaveAsTemplateDialog from '@/components/SaveAsTemplateDialog.vue';
import SearchDialog from '@/components/SearchDialog.vue';
import { CODEX_REF_EVENT, type CodexRefOpenDetail } from '@/editor/extensions/codex-ref';
import FootnoteDialog from '@/components/FootnoteDialog.vue';
import SplitSecondaryPane from '@/components/SplitSecondaryPane.vue';
import { useCodexStore } from '@/stores/codex';
import { useMediaStore } from '@/stores/media';
import { ipc } from '@/services/ipc';
import FindReplaceBar from '@/components/FindReplaceBar.vue';
import GoalProgress from '@/components/GoalProgress.vue';
import ProjectViewToggle from '@/components/ProjectViewToggle.vue';
import CorkboardView from '@/views/CorkboardView.vue';
import OutlinerView from '@/views/OutlinerView.vue';
import CodexView from '@/views/CodexView.vue';
import ScriveningsView from '@/components/ScriveningsView.vue';
import TipTapEditor from '@/editor/TipTapEditor.vue';
import EditorToolbar from '@/editor/EditorToolbar.vue';
import AiInlinePanel from '@/components/AiInlinePanel.vue';
import ValidationDialog from '@/components/ValidationDialog.vue';
import DictationOverlay from '@/components/DictationOverlay.vue';
import ReadAloudBar from '@/components/ReadAloudBar.vue';
import { useCapability } from '@/composables/useCapability';
import { useDictation } from '@/composables/useDictation';
import { useReadAloud } from '@/composables/useReadAloud';
import { findMatches } from '@/composables/useProseMirrorSearch';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const docStore = useDocumentStore();
const uiStore = useUiStore();
const { run } = useIpcError();

const focusMode = computed(() => uiStore.focusMode);
const typewriterEnabled = computed(() => uiStore.typewriterMode);
const viewMode = computed<ProjectViewMode>(() =>
  project.value ? uiStore.getProjectView(project.value.id) : 'editor',
);

function toggleFocus() {
  uiStore.toggleFocusMode();
}

const splitSecondaryId = computed<string | null>(() =>
  project.value ? uiStore.getSplitSecondary(project.value.id) : null,
);
const splitMode = computed(() => splitSecondaryId.value !== null);

function toggleSplit() {
  if (!project.value) return;
  if (splitMode.value) {
    uiStore.setSplitSecondary(project.value.id, null);
    return;
  }
  // Default to the first available doc that isn't the primary selection.
  const fallback = docStore.documents.find(
    (d) => d.docType !== 'folder' && d.id !== docStore.selectedId,
  );
  uiStore.setSplitSecondary(project.value.id, fallback?.id ?? '');
}

function onSecondaryIdChange(v: string | null) {
  if (!project.value) return;
  uiStore.setSplitSecondary(project.value.id, v);
}

function changeViewMode(mode: ProjectViewMode) {
  if (project.value) uiStore.setProjectView(project.value.id, mode);
}

function navigateDoc(delta: 1 | -1) {
  const ids = docStore.documents.map((d) => d.id);
  if (ids.length === 0) return;
  const current = docStore.selectedId;
  const idx = current ? ids.indexOf(current) : -1;
  const next = idx === -1 ? 0 : (idx + delta + ids.length) % ids.length;
  docStore.select(ids[next]);
}

const projectId = computed(() => String(route.params.id));
const project = computed(() => projectStore.projects.find((p) => p.id === projectId.value) ?? null);
const readOnly = computed(() => project.value?.status === 'archived');

const { selected, saveState, lastSavedAt, wordCount, totalWordCount } = storeToRefs(docStore);

const editorContent = ref('');
const editorContentJson = ref<string | null>(null);
const editorRef = ref<InstanceType<typeof TipTapEditor> | null>(null);
const editor = computed(() => editorRef.value?.editor ?? null);

const showExport = ref(false);
const showBibliography = ref(false);
const showCitationPicker = ref(false);
const showCodexPicker = ref(false);
const showFootnoteDialog = ref(false);
const editingFootnoteId = ref<string | null>(null);
const editingFootnoteContent = ref('');
const showSaveAsTemplate = ref(false);
const showSearch = ref(false);
const showValidation = ref(false);
const aiInline = useCapability('ai_inline');
const voiceDictation = useCapability('voice_dictation');
const voiceTts = useCapability('voice_tts');
const dictation = useDictation(editor);
const readAloud = useReadAloud(editor);

function onDictateKey(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && (e.key === 'M' || e.key === 'm')) {
    if (!voiceDictation.value || readOnly.value || !editor.value) return;
    e.preventDefault();
    dictation.toggle();
  }
}
const findVisible = ref(false);
const findMode = ref<'find' | 'replace'>('find');

/** Jump to a finding's excerpt in the editor (G-09 "ir al texto"). Best-effort:
 * exact match first, then the first few words. */
function onLocate(excerpt: string) {
  const ed = editor.value;
  if (!ed || !excerpt.trim()) return;
  let matches = findMatches(ed.state.doc, excerpt.trim(), false);
  if (matches.length === 0) {
    const short = excerpt.trim().split(/\s+/).slice(0, 6).join(' ');
    if (short) matches = findMatches(ed.state.doc, short, false);
  }
  if (matches.length > 0) {
    ed.chain().focus().setTextSelection(matches[0]).scrollIntoView().run();
  }
}
const sessionWordCount = computed(() => {
  const start = uiStore.sessionStartTotal;
  return start === null ? 0 : Math.max(0, totalWordCount.value - start);
});

const auto = useAutoSave(async () => {
  if (!selected.value) return;
  if (readOnly.value) return;
  await run(t('errors.saveDocument'), () =>
    docStore.save(selected.value!.id, {
      content: editorContent.value,
      contentJson: editorContentJson.value ?? undefined,
    }),
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
  // Snapshot the word count at load time so the inspector + app shell can
  // show "words written this session" without a roundtrip.
  uiStore.captureSessionStart(totalWordCount.value);
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
  editorContentJson.value = selected.value?.contentJson ?? null;
}

watch(selected, () => {
  // When the selection changes, flush pending save then load new content.
  void auto.flush().then(syncEditorFromSelection);
});

const mediaStore = useMediaStore();

watch(projectId, async (next, prev) => {
  if (next !== prev) {
    docStore.reset();
    mediaStore.reset();
    await loadProject();
  }
});

function onEditorInput(value: string) {
  editorContent.value = value;
  if (!readOnly.value) auto.trigger();
}

function onEditorJsonInput(value: string) {
  editorContentJson.value = value;
}

function onPickCitation(payload: { key: string; label: string }) {
  const ed = editor.value;
  if (!ed) return;
  ed.chain().focus().insertCitation({ citationKey: payload.key, label: payload.label }).run();
}

function onPickCodexRef(payload: { id: string; name: string }) {
  const ed = editor.value;
  if (!ed) return;
  ed.chain().focus().insertCodexRef({ entryId: payload.id, entryName: payload.name }).run();
}

/** Opens the OS file picker, reads the picked file, uploads it via the
 *  media service, and inserts an `<img data-media-id="…">` node. The
 *  NodeView resolves a Blob URL for display. */
async function onInsertImage() {
  if (!project.value || !editor.value) return;
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'] }],
    title: t('toolbar.insertImage'),
  });
  if (typeof picked !== 'string') return;
  const bytes = await readFile(picked);
  const mime = guessMimeFromPath(picked);
  const asset = await run(t('errors.uploadImage'), () =>
    ipc.uploadMedia({ projectId: project.value!.id, mime, bytes: Array.from(bytes) }),
  );
  if (!asset) return;
  const alt =
    picked
      .split(/[\\/]/)
      .pop()
      ?.replace(/\.[^.]+$/, '') ?? '';
  editor.value.chain().focus().insertImage({ mediaId: asset.id, alt }).run();
}

function onInsertFootnote() {
  editingFootnoteId.value = null;
  editingFootnoteContent.value = '';
  showFootnoteDialog.value = true;
}

function onSaveFootnote(content: string) {
  const ed = editor.value;
  if (!ed) return;
  if (editingFootnoteId.value) {
    ed.chain().focus().updateFootnote(editingFootnoteId.value, content).run();
  } else {
    const id = crypto.randomUUID();
    ed.chain().focus().insertFootnote({ id, content }).run();
  }
}

function onRemoveFootnote() {
  const ed = editor.value;
  const id = editingFootnoteId.value;
  if (!ed || !id) return;
  const { state } = ed;
  state.doc.descendants((node, pos) => {
    if (node.type.name !== 'footnote') return;
    if ((node.attrs as { id: string }).id !== id) return;
    ed.chain().focus().setNodeSelection(pos).deleteSelection().run();
  });
}

function guessMimeFromPath(path: string): string {
  const ext = path.toLowerCase().split('.').pop();
  switch (ext) {
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg';
    case 'png':
      return 'image/png';
    case 'gif':
      return 'image/gif';
    case 'webp':
      return 'image/webp';
    case 'svg':
      return 'image/svg+xml';
    default:
      return 'application/octet-stream';
  }
}

/** Cross-ref click from the editor: switch to the codex view and ask the
 *  store to surface the entry. The store may not have loaded yet (first
 *  visit to the project), so we await `loadFor` first. */
const codexStore = useCodexStore();
async function onCodexRefClick(e: Event) {
  const detail = (e as CustomEvent<CodexRefOpenDetail>).detail;
  if (!detail?.id || !project.value) return;
  if (project.value) uiStore.setProjectView(project.value.id, 'codex');
  await codexStore.loadFor(project.value.id);
  // The CodexView is bound to `projectId` and reads from the store; just
  // switching views is enough to surface the entry in the grid. Opening
  // the edit dialog directly would require coupling we don't need yet —
  // the user can click the card if they want to edit.
  const entry = codexStore.byId.get(detail.id);
  if (!entry) {
    tracingWarnMissing(detail.id);
  }
}

function tracingWarnMissing(id: string) {
  // Stale ref to a deleted entry — leave a console hint for the user.
  console.warn(`[codex] cross-ref points to missing entry ${id}`);
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

async function onSynopsisChange(synopsis: string | null) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setSynopsis(selected.value!.id, synopsis));
}

async function onOutlinerTitle(payload: { id: string; title: string }) {
  if (readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.save(payload.id, { title: payload.title }));
}

async function onOutlinerSynopsis(payload: { id: string; synopsis: string | null }) {
  if (readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setSynopsis(payload.id, payload.synopsis));
}

async function onOutlinerStatus(payload: { id: string; status: DocumentStatus }) {
  if (readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setStatus(payload.id, payload.status));
}

async function onProjectGoalChange(goal: number | null) {
  if (!project.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => projectStore.setGoal(project.value!.id, goal));
}

useShortcuts({
  flushSave: () => {
    void auto.flush();
  },
  newChapter: () => {
    if (!readOnly.value) onCreate('chapter');
  },
  searchProject: () => {
    showSearch.value = true;
  },
  findInDocument: () => {
    findMode.value = 'find';
    findVisible.value = true;
  },
  replaceInDocument: () => {
    if (readOnly.value) return;
    findMode.value = 'replace';
    findVisible.value = true;
  },
  prevDocument: () => navigateDoc(-1),
  nextDocument: () => navigateDoc(1),
  focusMode: () => toggleFocus(),
});

useTypewriterScroll(editor, typewriterEnabled);

function onSearchJump(documentId: string) {
  docStore.select(documentId);
}

function onFootnoteClickFromEditor(e: Event) {
  const detail = (e as CustomEvent<{ id: string; content: string }>).detail;
  editingFootnoteId.value = detail.id;
  editingFootnoteContent.value = detail.content;
  showFootnoteDialog.value = true;
}

onMounted(() => {
  loadProject();
  window.addEventListener(CODEX_REF_EVENT, onCodexRefClick as EventListener);
  window.addEventListener('draffity:open-footnote', onFootnoteClickFromEditor as EventListener);
  window.addEventListener('keydown', onDictateKey);
});

onBeforeUnmount(() => {
  window.removeEventListener(CODEX_REF_EVENT, onCodexRefClick as EventListener);
  window.removeEventListener('draffity:open-footnote', onFootnoteClickFromEditor as EventListener);
  window.removeEventListener('keydown', onDictateKey);
});
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
      <ProjectViewToggle :model-value="viewMode" @update:model-value="changeViewMode" />
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
        v-tooltip.bottom="t('split.toggle')"
        icon="pi pi-clone"
        text
        severity="secondary"
        size="small"
        :aria-label="t('split.toggle')"
        :aria-pressed="splitMode"
        @click="toggleSplit"
      />
      <Button
        icon="pi pi-book"
        text
        severity="secondary"
        size="small"
        :aria-label="t('bibliography.openButton')"
        @click="showBibliography = true"
      />
      <Button
        icon="pi pi-bookmark"
        text
        severity="secondary"
        size="small"
        :aria-label="t('saveAsTemplate.openButton')"
        @click="showSaveAsTemplate = true"
      />
      <Button
        v-if="aiInline"
        v-tooltip.bottom="t('ai.validators.title')"
        icon="pi pi-verified"
        text
        severity="secondary"
        size="small"
        :aria-label="t('ai.validators.title')"
        @click="showValidation = true"
      />
      <Button
        v-if="voiceDictation"
        v-tooltip.bottom="t('voice.dictation.button')"
        icon="pi pi-microphone"
        text
        :severity="dictation.phase.value === 'recording' ? 'danger' : 'secondary'"
        size="small"
        :disabled="readOnly"
        :aria-label="t('voice.dictation.button')"
        :aria-pressed="dictation.phase.value !== 'idle'"
        @click="dictation.toggle()"
      />
      <Button
        v-if="voiceTts"
        v-tooltip.bottom="t('voice.readAloud.button')"
        icon="pi pi-volume-up"
        text
        :severity="readAloud.phase.value !== 'idle' ? 'primary' : 'secondary'"
        size="small"
        :aria-label="t('voice.readAloud.button')"
        :aria-pressed="readAloud.phase.value !== 'idle'"
        @click="readAloud.toggle()"
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
    <BibliographyDialog v-model:visible="showBibliography" :project-id="project.id" />
    <CitationPickerDialog
      v-model:visible="showCitationPicker"
      :project-id="project.id"
      @pick="onPickCitation"
    />
    <CodexRefPickerDialog
      v-model:visible="showCodexPicker"
      :project-id="project.id"
      @pick="onPickCodexRef"
    />
    <FootnoteDialog
      v-model:visible="showFootnoteDialog"
      :initial-content="editingFootnoteContent"
      @save="onSaveFootnote"
      @remove="onRemoveFootnote"
    />
    <SaveAsTemplateDialog v-model:visible="showSaveAsTemplate" :project-id="project.id" />
    <SearchDialog v-model:visible="showSearch" :project-id="project.id" @jump="onSearchJump" />
    <ValidationDialog
      v-model:visible="showValidation"
      :project-id="project.id"
      :document-id="docStore.selectedId"
      @locate="onLocate"
    />
    <DictationOverlay
      :phase="dictation.phase.value"
      :level="dictation.level.value"
      @stop="dictation.stopAndInsert"
      @cancel="dictation.cancel"
    />
    <ReadAloudBar
      :phase="readAloud.phase.value"
      :speed="readAloud.speed.value"
      :speeds="readAloud.speeds"
      @pause="readAloud.pause"
      @resume="readAloud.resume"
      @stop="readAloud.stop"
      @skip="readAloud.skip"
      @update:speed="readAloud.setSpeed"
    />

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
        <template v-if="viewMode === 'editor'">
          <EditorToolbar
            v-if="selected?.docType !== 'folder'"
            :editor="editor"
            :disabled="readOnly"
            @open-citation-picker="showCitationPicker = true"
            @open-codex-picker="showCodexPicker = true"
            @insert-image="onInsertImage"
            @insert-footnote="onInsertFootnote"
          />
          <FindReplaceBar
            v-if="selected?.docType !== 'folder'"
            v-model:visible="findVisible"
            :editor="editor"
            :mode="findMode"
            :read-only="readOnly"
          />
          <div class="flex-1 min-h-0 bg-surface-0 dark:bg-surface-950">
            <div
              v-if="!selected"
              class="h-full flex items-center justify-center text-sm opacity-60"
            >
              {{ t('project.noSelection') }}
            </div>
            <ScriveningsView
              v-else-if="selected.docType === 'folder'"
              :folder="selected"
              :documents="docStore.documents"
            />
            <Splitter v-else-if="splitMode" class="h-full !rounded-none !border-0">
              <SplitterPanel :size="50" :min-size="25" class="!min-w-0">
                <TipTapEditor
                  ref="editorRef"
                  :model-value="editorContent"
                  :model-value-json="editorContentJson"
                  :editable="!readOnly"
                  :placeholder="t('project.untitled')"
                  @update:model-value="onEditorInput"
                  @update:model-value-json="onEditorJsonInput"
                />
              </SplitterPanel>
              <SplitterPanel :size="50" :min-size="25" class="!min-w-0">
                <SplitSecondaryPane
                  :project-id="project.id"
                  :primary-doc-id="docStore.selectedId"
                  :secondary-doc-id="splitSecondaryId"
                  :read-only="readOnly"
                  @update:secondary-doc-id="onSecondaryIdChange"
                  @close="toggleSplit"
                />
              </SplitterPanel>
            </Splitter>
            <TipTapEditor
              v-else
              ref="editorRef"
              :model-value="editorContent"
              :model-value-json="editorContentJson"
              :editable="!readOnly"
              :placeholder="t('project.untitled')"
              @update:model-value="onEditorInput"
              @update:model-value-json="onEditorJsonInput"
            />
          </div>
          <AiInlinePanel
            :editor="editor"
            :project-id="project.id"
            :doc-id="docStore.selectedId"
            :disabled="readOnly"
          />
        </template>
        <CorkboardView
          v-else-if="viewMode === 'corkboard'"
          :documents="docStore.documents"
          :selected-id="docStore.selectedId"
          @select="onSelect"
        />
        <OutlinerView
          v-else-if="viewMode === 'outliner'"
          :documents="docStore.documents"
          :selected-id="docStore.selectedId"
          :read-only="readOnly"
          @select="onSelect"
          @update-title="onOutlinerTitle"
          @update-synopsis="onOutlinerSynopsis"
          @update-status="onOutlinerStatus"
        />
        <CodexView v-else :project-id="project.id" :read-only="readOnly" />
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
          @synopsis-change="onSynopsisChange"
        />
      </SplitterPanel>
    </Splitter>
  </div>
</template>
