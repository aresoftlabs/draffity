<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import Slider from 'primevue/slider';
import Select from 'primevue/select';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { DocumentStatus, DocumentType } from '@draffity/shared-types';
import type { MenuItem } from 'primevue/menuitem';

import { useProjectStore } from '@/stores/project';
import { useDocumentStore, type ReorderOp } from '@/stores/document';
import { useUiStore, type ProjectViewMode } from '@/stores/ui';
import { useToast } from 'primevue/usetoast';
import { useIpcError } from '@/composables/useIpcError';
import { useEditorAutoSave } from '@/composables/useEditorAutoSave';
import { useShortcuts } from '@/composables/useShortcuts';
import { useTypewriterScroll } from '@/composables/useTypewriterScroll';
import { registerCommands } from '@/composables/useCommandRegistry';

import Binder from '@/components/Binder.vue';
import CollectionsPanel from '@/components/CollectionsPanel.vue';
import Inspector from '@/components/Inspector.vue';
import LabelManagerDialog from '@/components/LabelManagerDialog.vue';
import CustomFieldsManagerDialog from '@/components/CustomFieldsManagerDialog.vue';
import AppStatusBar from '@/components/AppStatusBar.vue';
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
import { useLabelStore } from '@/stores/labels';
import { useCustomFieldStore } from '@/stores/customFields';
import { ipc, type AiStatus, type VoiceStatus } from '@/services/ipc';
import FindReplaceBar from '@/components/FindReplaceBar.vue';
import AppRail from '@/components/AppRail.vue';
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
import VoiceNotesDialog from '@/components/VoiceNotesDialog.vue';
import { useEditorSettings } from '@/composables/useEditorSettings';
import { useDictation } from '@/composables/useDictation';
import { useReadAloud } from '@/composables/useReadAloud';
import { findMatches } from '@/composables/useProseMirrorSearch';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const docStore = useDocumentStore();
const uiStore = useUiStore();
const labelStore = useLabelStore();
const customFieldStore = useCustomFieldStore();
const { run } = useIpcError();
const toast = useToast();

const labelManagerVisible = ref(false);
const fieldsManagerVisible = ref(false);

const focusMode = computed(() => uiStore.focusMode);
const compositionMode = computed(() => uiStore.compositionMode);
const typewriterEnabled = computed(() => uiStore.typewriterMode);
const { autosaveMs, paperWidthCh, compositionBg, fadeLevel } = useEditorSettings();
const fadeOptions = computed(() => [
  { value: 'none' as const, label: t('composition.fadeNone') },
  { value: 'paragraph' as const, label: t('composition.fadeParagraph') },
]);
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

// The editor content is bound to the id of the document it belongs to, so an
// autosave always targets that document â€” never the live selection (AUD-01).
const editorDoc = useEditorAutoSave({
  persist: (id, payload) => run(t('errors.saveDocument'), () => docStore.save(id, payload)),
  readOnly: () => readOnly.value,
  delay: () => autosaveMs.value,
});
const editorContent = editorDoc.content;
const editorContentJson = editorDoc.contentJson;
const editorRef = ref<InstanceType<typeof TipTapEditor> | null>(null);
const editor = computed(() => editorRef.value?.editor ?? null);

// Apply the Linguistic Focus overlay (J-06) whenever the editor becomes
// available or the persisted toggle changes. The decoration plugin recomputes
// on every doc change on its own, so this only flips the on/off flag.
watch(
  [editor, () => uiStore.linguisticFocus, () => uiStore.linguisticExtraWords],
  ([ed, enabled, extraWords]) => {
    ed?.commands.setLinguisticFocus(!!enabled, { extraWords: (extraWords as string[]) ?? [] });
  },
  { immediate: true, deep: true },
);

// Repetition heatmap (J-08): apply on editor ready / toggle change.
watch(
  [editor, () => uiStore.repetitionHeatmap],
  ([ed, enabled]) => {
    ed?.commands.setRepetitionHeatmap(!!enabled);
  },
  { immediate: true },
);

// Paragraph fade (K-08): only in composition mode with fadeLevel = paragraph.
watch(
  [editor, compositionMode, () => fadeLevel.value],
  ([ed]) => {
    ed?.commands.setParagraphFade(compositionMode.value && fadeLevel.value === 'paragraph');
  },
  { immediate: true },
);

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
const aiStatus = ref<AiStatus | null>(null);
const voiceStatus = ref<VoiceStatus | null>(null);
// Availability comes from real prerequisites now (BYOK key / installed
// binaries), real prerequisites (BYOK key / installed binaries). Voice notes are local storage => always on.. Voice notes are local storage â†’ always on.
const aiInline = computed(() => aiStatus.value?.available ?? false);
const voiceDictation = computed(() => voiceStatus.value?.dictationAvailable ?? false);
const voiceTts = computed(() => voiceStatus.value?.ttsAvailable ?? false);
const showVoiceNotes = ref(false);
function notifyVoiceError(message: string) {
  toast.add({ severity: 'error', summary: t('voice.error'), detail: message, life: 6000 });
}
const dictation = useDictation(editor, { onError: notifyVoiceError });
const readAloud = useReadAloud(editor, { onError: notifyVoiceError });

const actionMenu = ref<{ toggle: (e: Event) => void } | null>(null);
const actionItems = computed<MenuItem[]>(() => [
  {
    label: t('actions.export'),
    icon: 'pi pi-download',
    command: () => {
      showExport.value = true;
    },
  },
  {
    label: t('bibliography.openButton'),
    icon: 'pi pi-book',
    command: () => {
      showBibliography.value = true;
    },
  },
  {
    label: t('saveAsTemplate.openButton'),
    icon: 'pi pi-bookmark',
    command: () => {
      showSaveAsTemplate.value = true;
    },
  },
  {
    label: t('ai.validators.title'),
    icon: 'pi pi-verified',
    visible: Boolean(aiInline.value),
    command: () => {
      showValidation.value = true;
    },
  },
  { separator: true },
  {
    label: t('voice.notes.button'),
    icon: 'pi pi-comment',
    command: () => {
      showVoiceNotes.value = true;
    },
  },
]);

function onDictateKey(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && (e.key === 'M' || e.key === 'm')) {
    if (!voiceDictation.value || readOnly.value || !editor.value) return;
    e.preventDefault();
    dictation.toggle();
  }
}

/** Composition mode (K-09): Ctrl+Shift+F11 toggles, Esc exits. */
function onCompositionKey(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && e.key === 'F11') {
    e.preventDefault();
    uiStore.toggleCompositionMode();
  } else if (e.key === 'Escape' && uiStore.compositionMode) {
    uiStore.toggleCompositionMode();
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

async function loadProject() {
  if (!projectStore.projects.length) {
    await run(t('errors.loadProjects'), () => projectStore.loadAll());
  }
  if (!project.value) {
    router.replace({ name: 'dashboard' });
    return;
  }
  await run(t('errors.loadDocuments'), () => docStore.loadFor(projectId.value));
  await run(t('labels.error'), () => labelStore.loadFor(projectId.value));
  await run(t('customFields.error'), () => customFieldStore.loadFor(projectId.value));
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
  // load() flushes the previous document's pending save before swapping in the
  // new content, so a fast selection change can't persist to the wrong doc.
  void editorDoc.load(selected.value);
}

watch(selected, syncEditorFromSelection);

const mediaStore = useMediaStore();

watch(projectId, async (next, prev) => {
  if (next !== prev) {
    docStore.reset();
    mediaStore.reset();
    labelStore.reset();
    customFieldStore.reset();
    await loadProject();
  }
});

function onEditorInput(value: string) {
  editorDoc.onContent(value);
}

function onEditorJsonInput(value: string) {
  editorDoc.onContentJson(value);
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
 *  media service, and inserts an `<img data-media-id="â€¦">` node. The
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
  // the edit dialog directly would require coupling we don't need yet â€”
  // the user can click the card if they want to edit.
  const entry = codexStore.byId.get(detail.id);
  if (!entry) {
    tracingWarnMissing(detail.id);
  }
}

function tracingWarnMissing(id: string) {
  // Stale ref to a deleted entry â€” leave a console hint for the user.
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

async function onLabelsChange(labelIds: string[]) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setLabels(selected.value!.id, labelIds));
}

async function onMetadataChange(fieldId: string, value: string | null) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () =>
    docStore.setMetadata(selected.value!.id, fieldId, value),
  );
}

async function onResearchChange(isResearch: boolean) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => docStore.setResearch(selected.value!.id, isResearch));
}

async function onMatterChange(isFront: boolean, isBack: boolean) {
  if (!selected.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () =>
    docStore.setMatter(selected.value!.id, isFront, isBack),
  );
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

async function onProjectDeadlineChange(deadline: number | null) {
  if (!project.value || readOnly.value) return;
  await run(t('errors.saveDocument'), () => projectStore.setDeadline(project.value!.id, deadline));
}

useShortcuts({
  flushSave: () => {
    void editorDoc.flush();
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

let offProjectCmds: (() => void) | null = null;

onMounted(() => {
  loadProject();
  ipc
    .getAiStatus()
    .then((s) => {
      aiStatus.value = s;
    })
    .catch((e) => {
      console.error('[project]', 'aiStatus', e);
    });
  ipc
    .getVoiceStatus()
    .then((s) => {
      voiceStatus.value = s;
    })
    .catch((e) => {
      console.error('[project]', 'voiceStatus', e);
    });
  window.addEventListener(CODEX_REF_EVENT, onCodexRefClick as EventListener);
  window.addEventListener('draffity:open-footnote', onFootnoteClickFromEditor as EventListener);
  window.addEventListener('keydown', onDictateKey);
  window.addEventListener('keydown', onCompositionKey);
  offProjectCmds = registerCommands([
    {
      id: 'project.search',
      label: t('command.searchProject'),
      group: t('command.groupProject'),
      icon: 'pi pi-search',
      run: () => {
        showSearch.value = true;
      },
    },
    {
      id: 'project.focus',
      label: t('command.toggleFocus'),
      group: t('command.groupProject'),
      icon: 'pi pi-expand',
      run: () => {
        toggleFocus();
      },
    },
    {
      id: 'project.export',
      label: t('command.exportManuscript'),
      group: t('command.groupProject'),
      icon: 'pi pi-file-export',
      run: () => {
        showExport.value = true;
      },
    },
    {
      id: 'project.newChapter',
      label: t('command.newChapter'),
      group: t('command.groupProject'),
      icon: 'pi pi-plus',
      keywords: ['capitulo', 'chapter'],
      run: () => {
        if (!readOnly.value) onCreate('chapter');
      },
    },
  ]);
});

onBeforeUnmount(() => {
  window.removeEventListener(CODEX_REF_EVENT, onCodexRefClick as EventListener);
  window.removeEventListener('draffity:open-footnote', onFootnoteClickFromEditor as EventListener);
  window.removeEventListener('keydown', onDictateKey);
  window.removeEventListener('keydown', onCompositionKey);
  offProjectCmds?.();
});
</script>

<template>
  <div v-if="project" class="flex-1 flex min-h-0">
    <AppRail
      v-if="!focusMode && !compositionMode"
      :model-value="viewMode"
      @update:model-value="changeViewMode"
      @search="showSearch = true"
    />
    <div class="flex-1 flex flex-col min-h-0">
      <!-- Single top bar: project-contextual actions are teleported into the
           global AppTopBar. ProjectView keeps the handlers; AppTopBar only
           hosts them. Hidden in composition mode (the top bar is hidden too). -->
      <Teleport v-if="!compositionMode" to="#topbar-project-actions">
        <Tag v-if="readOnly" :value="t('dashboard.readOnly')" severity="secondary" class="mr-1" />
        <Button
          v-tooltip.bottom="t('project.focusMode')"
          :icon="focusMode ? 'pi pi-window-minimize' : 'pi pi-arrows-alt'"
          text
          rounded
          severity="secondary"
          size="small"
          :aria-label="t('project.focusMode')"
          :aria-pressed="focusMode"
          @click="toggleFocus"
        />
        <Button
          v-tooltip.bottom="t('composition.enter')"
          icon="pi pi-desktop"
          text
          rounded
          severity="secondary"
          size="small"
          :aria-label="t('composition.enter')"
          @click="uiStore.toggleCompositionMode()"
        />
        <Button
          v-tooltip.bottom="t('split.toggle')"
          icon="pi pi-clone"
          text
          rounded
          severity="secondary"
          size="small"
          :aria-label="t('split.toggle')"
          :aria-pressed="splitMode"
          @click="toggleSplit"
        />
        <Button
          v-tooltip.bottom="t('project.moreActions')"
          icon="pi pi-ellipsis-v"
          text
          rounded
          severity="secondary"
          size="small"
          :aria-label="t('project.moreActions')"
          aria-haspopup="true"
          @click="actionMenu?.toggle($event)"
        />
        <Menu ref="actionMenu" :model="actionItems" popup />
      </Teleport>

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
        :waveform="dictation.waveform.value"
        :elapsed-ms="dictation.elapsedMs.value"
        :is-silent="dictation.isSilent.value"
        :progress="dictation.progress.value"
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
      <VoiceNotesDialog v-model:visible="showVoiceNotes" :project-id="project.id" />

      <Splitter
        class="flex-1 !rounded-none !border-0 min-h-0"
        :pt="{
          gutter: { class: 'bg-surface-200 dark:bg-surface-700' },
        }"
        style-class="h-full"
      >
        <SplitterPanel
          v-if="!focusMode && !compositionMode"
          :size="22"
          :min-size="14"
          class="!min-w-0"
        >
          <div class="h-full flex flex-col min-h-0">
            <div class="flex-1 min-h-0 overflow-auto">
              <Binder
                :documents="docStore.documents"
                :selected-id="docStore.selectedId"
                :labels="labelStore.labels"
                :read-only="readOnly"
                @select="onSelect"
                @create="onCreate"
                @reorder="onReorder"
              />
            </div>
            <CollectionsPanel
              :project-id="project.id"
              :current-doc-id="docStore.selectedId"
              :read-only="readOnly"
              @select="onSelect"
            />
          </div>
        </SplitterPanel>

        <SplitterPanel
          :size="focusMode || compositionMode ? 100 : 56"
          :min-size="30"
          class="!min-w-0 flex flex-col"
        >
          <div
            v-if="readOnly"
            class="px-4 py-2 text-xs italic bg-amber-100 dark:bg-amber-900/30 text-amber-900 dark:text-amber-200 border-b border-amber-300 dark:border-amber-800"
          >
            {{ t('project.readOnlyBanner') }}
          </div>
          <template v-if="viewMode === 'editor'">
            <EditorToolbar
              v-if="selected?.docType !== 'folder' && !compositionMode"
              :editor="editor"
              :disabled="readOnly"
              :linguistic-focus-active="uiStore.linguisticFocus"
              :repetition-active="uiStore.repetitionHeatmap"
              @open-citation-picker="showCitationPicker = true"
              @open-codex-picker="showCodexPicker = true"
              @insert-image="onInsertImage"
              @insert-footnote="onInsertFootnote"
              @toggle-linguistic-focus="uiStore.toggleLinguisticFocus()"
              @toggle-repetition="uiStore.toggleRepetitionHeatmap()"
            />
            <FindReplaceBar
              v-if="selected?.docType !== 'folder' && !compositionMode"
              v-model:visible="findVisible"
              :editor="editor"
              :mode="findMode"
              :read-only="readOnly"
            />
            <div
              class="flex-1 min-h-0 bg-surface-0 dark:bg-surface-950"
              :style="
                compositionMode && compositionBg ? { backgroundColor: compositionBg } : undefined
              "
            >
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
                :paper-width-ch="compositionMode ? paperWidthCh : 0"
                @update:model-value="onEditorInput"
                @update:model-value-json="onEditorJsonInput"
              />
            </div>
            <AiInlinePanel
              v-if="!compositionMode"
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
            :custom-fields="customFieldStore.fields"
            :read-only="readOnly"
            @select="onSelect"
            @update-title="onOutlinerTitle"
            @update-synopsis="onOutlinerSynopsis"
            @update-status="onOutlinerStatus"
          />
          <CodexView v-else :project-id="project.id" :read-only="readOnly" />
        </SplitterPanel>

        <SplitterPanel
          v-if="!focusMode && !compositionMode"
          :size="22"
          :min-size="14"
          class="!min-w-0"
        >
          <Inspector
            :doc="selected"
            :word-count-here="wordCount"
            :word-count-total="totalWordCount"
            :session-word-count="sessionWordCount"
            :labels="labelStore.labels"
            :custom-fields="customFieldStore.fields"
            :reading-wpm="uiStore.readingWpm"
            :read-only="readOnly"
            @snapshot-restored="onSnapshotRestored"
            @status-change="onStatusChange"
            @tags-change="onTagsChange"
            @labels-change="onLabelsChange"
            @manage-labels="labelManagerVisible = true"
            @metadata-change="onMetadataChange"
            @manage-fields="fieldsManagerVisible = true"
            @research-change="onResearchChange"
            @matter-change="onMatterChange"
            @goal-change="onDocGoalChange"
            @synopsis-change="onSynopsisChange"
          />
        </SplitterPanel>
      </Splitter>

      <AppStatusBar
        v-if="!focusMode && !compositionMode"
        :total-word-count="totalWordCount"
        :save-state="saveState"
        :last-saved-at="lastSavedAt"
        :project-goal="project.goalWords ?? null"
        :project-deadline="project.deadline ?? null"
        :session-words="sessionWordCount"
        :session-goal="uiStore.sessionGoal"
        :read-only="readOnly"
        :voice-tts="voiceTts"
        :voice-dictation="voiceDictation"
        :read-aloud-active="readAloud.phase.value !== 'idle'"
        :dictation-active="dictation.phase.value !== 'idle'"
        @update:project-goal="onProjectGoalChange"
        @update:project-deadline="onProjectDeadlineChange"
        @update:session-goal="uiStore.setSessionGoal"
        @toggle-read-aloud="readAloud.toggle()"
        @toggle-dictation="dictation.toggle()"
      />

      <!-- Composition mode control bar (K-09): hidden until hover at the top. -->
      <div v-if="compositionMode" class="composition-bar">
        <div
          class="composition-bar-inner flex items-center gap-4 px-4 py-2 bg-surface-0/95 dark:bg-surface-950/95 border-b border-surface-200 dark:border-surface-700 backdrop-blur"
        >
          <Button
            icon="pi pi-times"
            text
            size="small"
            :label="t('composition.exit')"
            @click="uiStore.toggleCompositionMode()"
          />
          <span class="text-xs opacity-60 font-mono">{{ wordCount }} · {{ totalWordCount }}</span>
          <span class="flex-1" />
          <label class="flex items-center gap-2 text-xs opacity-70">
            {{ t('composition.paperWidth') }}
            <Slider v-model="paperWidthCh" :min="50" :max="140" :step="5" class="!w-32" />
            <span class="font-mono w-8">{{ paperWidthCh }}</span>
          </label>
          <label class="flex items-center gap-2 text-xs opacity-70">
            {{ t('composition.background') }}
            <input
              type="color"
              :value="compositionBg || '#ffffff'"
              class="w-7 h-7 rounded border-0 bg-transparent cursor-pointer"
              :aria-label="t('composition.background')"
              @input="(e) => (compositionBg = (e.target as HTMLInputElement).value)"
            />
            <Button
              v-if="compositionBg"
              icon="pi pi-times"
              text
              size="small"
              :pt="{ root: { class: '!w-5 !h-5 !p-0' } }"
              :aria-label="t('composition.resetBackground')"
              @click="compositionBg = ''"
            />
          </label>
          <label class="flex items-center gap-2 text-xs opacity-70">
            {{ t('composition.fade') }}
            <Select
              v-model="fadeLevel"
              :options="fadeOptions"
              option-label="label"
              option-value="value"
              size="small"
              class="!text-xs"
            />
          </label>
        </div>
      </div>

      <LabelManagerDialog v-model:visible="labelManagerVisible" :project-id="projectId" />
      <CustomFieldsManagerDialog v-model:visible="fieldsManagerVisible" :project-id="projectId" />
    </div>
  </div>
</template>

<style scoped>
/* Composition control bar reveals on hover at the very top of the screen. */
.composition-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 60;
  height: 8px;
}
.composition-bar-inner {
  transform: translateY(-100%);
  transition: transform 0.2s ease;
}
.composition-bar:hover .composition-bar-inner,
.composition-bar:focus-within .composition-bar-inner {
  transform: translateY(0);
}
</style>
