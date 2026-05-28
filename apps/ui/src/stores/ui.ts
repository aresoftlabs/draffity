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

export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>(getStoredTheme());
  const binderCollapsed = ref(loadBool('binderCollapsed', false));
  const inspectorCollapsed = ref(loadBool('inspectorCollapsed', false));
  const focusMode = ref(false);
  // One-shot flag set by onboarding to ask the dashboard to open the
  // NewProjectWizard automatically. The dashboard clears it after acting.
  const pendingNewProject = ref(false);

  // Writing session — words written since the current project was opened.
  // `sessionStartTotal` is captured by ProjectView on load; the live count
  // comes from the document store. `sessionGoal` is persisted so it
  // survives restarts.
  const sessionGoal = ref<number | null>(loadNumber('sessionGoal'));
  const sessionStartTotal = ref<number | null>(null);

  watch(binderCollapsed, (v) => saveBool('binderCollapsed', v));
  watch(inspectorCollapsed, (v) => saveBool('inspectorCollapsed', v));
  watch(sessionGoal, (v) => saveNumber('sessionGoal', v));

  function setTheme(mode: ThemeMode) {
    theme.value = mode;
    applyTheme(mode);
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

  return {
    theme,
    binderCollapsed,
    inspectorCollapsed,
    focusMode,
    pendingNewProject,
    sessionGoal,
    sessionStartTotal,
    setTheme,
    setLocale,
    toggleBinder,
    toggleInspector,
    toggleFocusMode,
    requestNewProject,
    consumeNewProjectRequest,
    captureSessionStart,
    setSessionGoal,
    clearSession,
  };
});
