export type ThemeMode = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'draffity.theme';
const DARK_CLASS = 'app-dark';

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function resolveDark(mode: ThemeMode): boolean {
  if (mode === 'system') return systemPrefersDark();
  return mode === 'dark';
}

function applyDarkClass(isDark: boolean) {
  const html = document.documentElement;
  if (isDark) html.classList.add(DARK_CLASS);
  else html.classList.remove(DARK_CLASS);
}

export function getStoredTheme(): ThemeMode {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'light' || stored === 'dark' || stored === 'system') return stored;
  return 'system';
}

export function setTheme(mode: ThemeMode) {
  localStorage.setItem(STORAGE_KEY, mode);
  applyDarkClass(resolveDark(mode));
}

export function applyInitialTheme() {
  const mode = getStoredTheme();
  applyDarkClass(resolveDark(mode));

  // Respond to OS theme changes when in 'system' mode
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (getStoredTheme() === 'system') {
      applyDarkClass(systemPrefersDark());
    }
  });
}
