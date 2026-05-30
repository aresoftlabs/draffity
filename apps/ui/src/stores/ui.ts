import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { setTheme as applyTheme, getStoredTheme, type ThemeMode } from '@/styles/theme';
import { setLocale as applyLocale } from '@/locales';

const STORAGE_PREFIX = 'draffity.ui.';

function loadBool(key: string, fallback: boolean): boolean {
  if (typeof localStorage === 'undefined') return fallback;
  const raw = localStorage.getItem(STORAGE_PREFIX + key);
  if (raw === '1') return true;
  if (raw === '0') return false;
  return fallback;
}

function saveBool(key: string, value: boolean) {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORAGE_PREFIX + key, value ? '1' : '0');
}

function loadNumber(key: string): number | null {
  if (typeof localStorage === 'undefined') return null;
  const raw = localStorage.getItem(STORAGE_PREFIX + key);
  if (raw == null || raw === '') return null;
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : null;
}

function saveNumber(key: string, value: number | null) {
  if (typeof localStorage === 'undefined') return;
  if (value == null) localStorage.removeItem(STORAGE_PREFIX + key);
  else localStorage.setItem(STORAGE_PREFIX + key, String(value));
}

function loadJson<T>(key: string, fallback: T): T {
  if (typeof localStorage === 'undefined') return fallback;
  const raw = localStorage.getItem(STORAGE_PREFIX + key);
  if (!raw) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function saveJson<T>(key: string, value: T) {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(value));
}

export type ProjectViewMode = 'editor' | 'corkboard' | 'outliner' | 'codex';

export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>(getStoredTheme());
  const binderCollapsed = ref(loadBool('binderCollapsed', false));
  const inspectorCollapsed = ref(loadBool('inspectorCollapsed', false));
  const focusMode = ref(false);
  // Composition mode (K-08): distraction-free fullscreen writing surface.
  // Session-only like focusMode — it's a transient mode, not a preference.
  const compositionMode = ref(false);
  const typewriterMode = ref(loadBool('typewriterMode', false));
  // Linguistic Focus (J-06): highlight adverbs / passive voice / dialogue in
  // the editor. Persisted so the mode survives restarts.
  const linguisticFocus = ref(loadBool('linguisticFocus', false));
  // Extra words to flag as adverbs in Linguistic Focus (J-07, configurable).
  const linguisticExtraWords = ref<string[]>(loadJson('linguisticExtraWords', []));
  // Reading-speed for reading-time estimates (J-09), words per minute.
  const readingWpm = ref<number>(loadNumber('readingWpm') ?? 200);
  // Repetition heatmap (J-08): highlight over-used words/phrases in the editor.
  const repetitionHeatmap = ref(loadBool('repetitionHeatmap', false));
  // User-imported name lists for the generator (K-07). Treated as unisex pools.
  const customNameLists = ref<{ id: string; label: string; names: string[] }[]>(
    loadJson('customNameLists', []),
  );
  // One-shot flag set by onboarding to ask the dashboard to open the
  // NewProjectWizard automatically. The dashboard clears it after acting.
  const pendingNewProject = ref(false);

  // Writing session — words written since the current project was opened.
  // `sessionStartTotal` is captured by ProjectView on load; the live count
  // comes from the document store. `sessionGoal` is persisted so it
  // survives restarts.
  const sessionGoal = ref<number | null>(loadNumber('sessionGoal'));
  const sessionStartTotal = ref<number | null>(null);

  // View mode per project: Editor / Corkboard / Outliner. Keyed by
  // projectId so each project remembers how the user left it.
  const projectViewModes = ref<Record<string, ProjectViewMode>>(loadJson('projectViewModes', {}));

  // Split editor: secondary doc id per project. `null` means split mode
  // is off. When set, ProjectView renders a second editor pane alongside
  // the primary one and autosaves to this doc.
  const splitSecondaryIds = ref<Record<string, string | null>>(loadJson('splitSecondaryIds', {}));

  // Recently-opened docs in the secondary split pane, per project (K-10).
  // Most-recent first, capped — powers the per-pane bookmark chips.
  const splitBookmarks = ref<Record<string, string[]>>(loadJson('splitBookmarks', {}));

  watch(binderCollapsed, (v) => saveBool('binderCollapsed', v));
  watch(inspectorCollapsed, (v) => saveBool('inspectorCollapsed', v));
  watch(typewriterMode, (v) => saveBool('typewriterMode', v));
  watch(linguisticFocus, (v) => saveBool('linguisticFocus', v));
  watch(linguisticExtraWords, (v) => saveJson('linguisticExtraWords', v), { deep: true });
  watch(readingWpm, (v) => saveNumber('readingWpm', v));
  watch(repetitionHeatmap, (v) => saveBool('repetitionHeatmap', v));
  watch(customNameLists, (v) => saveJson('customNameLists', v), { deep: true });
  watch(sessionGoal, (v) => saveNumber('sessionGoal', v));
  watch(projectViewModes, (v) => saveJson('projectViewModes', v), { deep: true });
  watch(splitSecondaryIds, (v) => saveJson('splitSecondaryIds', v), { deep: true });
  watch(splitBookmarks, (v) => saveJson('splitBookmarks', v), { deep: true });

  function setTheme(mode: ThemeMode) {
    theme.value = mode;
    applyTheme(mode);
  }

  /** Effective dark state for the *current* theme. 'high-contrast' rides on
   *  top of dark; 'system' follows the OS preference. */
  function effectiveDark(): boolean {
    switch (theme.value) {
      case 'dark':
      case 'high-contrast':
        return true;
      case 'light':
        return false;
      default:
        return typeof window !== 'undefined' && typeof window.matchMedia === 'function'
          ? window.matchMedia('(prefers-color-scheme: dark)').matches
          : false;
    }
  }

  /** Quick top-bar toggle: only ever lands on a concrete light/dark theme.
   *  The full 4-way choice (incl. system / high-contrast) lives in Settings. */
  function toggleLightDark(): void {
    setTheme(effectiveDark() ? 'light' : 'dark');
  }

  function setLocale(locale: 'es' | 'en') {
    applyLocale(locale);
  }

  function toggleBinder() {
    binderCollapsed.value = !binderCollapsed.value;
  }

  function toggleInspector() {
    inspectorCollapsed.value = !inspectorCollapsed.value;
  }

  function toggleFocusMode() {
    focusMode.value = !focusMode.value;
  }

  function toggleCompositionMode() {
    compositionMode.value = !compositionMode.value;
  }

  function toggleTypewriterMode() {
    typewriterMode.value = !typewriterMode.value;
  }

  function toggleLinguisticFocus() {
    linguisticFocus.value = !linguisticFocus.value;
  }

  function toggleRepetitionHeatmap() {
    repetitionHeatmap.value = !repetitionHeatmap.value;
  }

  function addCustomNameList(label: string, names: string[]) {
    const clean = names.map((n) => n.trim()).filter(Boolean);
    if (!label.trim() || clean.length === 0) return;
    const id = `custom:${label.trim()}:${customNameLists.value.length}`;
    customNameLists.value = [...customNameLists.value, { id, label: label.trim(), names: clean }];
  }

  function removeCustomNameList(id: string) {
    customNameLists.value = customNameLists.value.filter((l) => l.id !== id);
  }

  function requestNewProject() {
    pendingNewProject.value = true;
  }

  function consumeNewProjectRequest(): boolean {
    if (pendingNewProject.value) {
      pendingNewProject.value = false;
      return true;
    }
    return false;
  }

  function captureSessionStart(currentTotal: number) {
    sessionStartTotal.value = currentTotal;
  }

  function setSessionGoal(value: number | null) {
    sessionGoal.value = value && value > 0 ? Math.floor(value) : null;
  }

  function clearSession() {
    sessionStartTotal.value = null;
  }

  function getProjectView(projectId: string): ProjectViewMode {
    return projectViewModes.value[projectId] ?? 'editor';
  }

  function setProjectView(projectId: string, mode: ProjectViewMode) {
    // Re-assign so the deep watcher picks it up.
    projectViewModes.value = { ...projectViewModes.value, [projectId]: mode };
  }

  function getSplitSecondary(projectId: string): string | null {
    return splitSecondaryIds.value[projectId] ?? null;
  }

  function getSplitBookmarks(projectId: string): string[] {
    return splitBookmarks.value[projectId] ?? [];
  }

  function pushSplitBookmark(projectId: string, docId: string) {
    const cur = splitBookmarks.value[projectId] ?? [];
    const next = [docId, ...cur.filter((id) => id !== docId)].slice(0, 6);
    splitBookmarks.value = { ...splitBookmarks.value, [projectId]: next };
  }

  function setSplitSecondary(projectId: string, docId: string | null) {
    splitSecondaryIds.value = {
      ...splitSecondaryIds.value,
      [projectId]: docId,
    };
  }

  return {
    theme,
    binderCollapsed,
    inspectorCollapsed,
    focusMode,
    compositionMode,
    typewriterMode,
    linguisticFocus,
    linguisticExtraWords,
    readingWpm,
    repetitionHeatmap,
    customNameLists,
    pendingNewProject,
    sessionGoal,
    sessionStartTotal,
    projectViewModes,
    setTheme,
    toggleLightDark,
    setLocale,
    toggleBinder,
    toggleInspector,
    toggleFocusMode,
    toggleCompositionMode,
    toggleTypewriterMode,
    toggleLinguisticFocus,
    toggleRepetitionHeatmap,
    addCustomNameList,
    removeCustomNameList,
    requestNewProject,
    consumeNewProjectRequest,
    captureSessionStart,
    setSessionGoal,
    clearSession,
    getProjectView,
    setProjectView,
    getSplitSecondary,
    setSplitSecondary,
    getSplitBookmarks,
    pushSplitBookmark,
  };
});
