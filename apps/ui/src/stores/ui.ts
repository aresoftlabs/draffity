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

export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>(getStoredTheme());
  const binderCollapsed = ref(loadBool('binderCollapsed', false));
  const inspectorCollapsed = ref(loadBool('inspectorCollapsed', false));
  const focusMode = ref(false);

  watch(binderCollapsed, (v) => saveBool('binderCollapsed', v));
  watch(inspectorCollapsed, (v) => saveBool('inspectorCollapsed', v));

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

  return {
    theme,
    binderCollapsed,
    inspectorCollapsed,
    focusMode,
    setTheme,
    setLocale,
    toggleBinder,
    toggleInspector,
    toggleFocusMode,
  };
});
