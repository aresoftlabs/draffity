export type ThemeMode = 'light' | 'dark' | 'high-contrast' | 'system';

const STORAGE_KEY = 'draffity.theme';
const DARK_CLASS = 'app-dark';
const HC_CLASS = 'app-high-contrast';

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

/**
 * Resolve the user's stored mode to a concrete (dark, high-contrast)
 * pair. High contrast always rides on top of dark so the WCAG-AAA
 * palette has the saturation the dark surface tokens expect.
 */
function resolveClasses(mode: ThemeMode): { dark: boolean; hc: boolean } {
  if (mode === 'high-contrast') return { dark: true, hc: true };
  if (mode === 'system') return { dark: systemPrefersDark(), hc: false };
  return { dark: mode === 'dark', hc: false };
}

function applyClasses(classes: { dark: boolean; hc: boolean }) {
  const html = document.documentElement;
  html.classList.toggle(DARK_CLASS, classes.dark);
  html.classList.toggle(HC_CLASS, classes.hc);
}

export function getStoredTheme(): ThemeMode {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (
    stored === 'light' ||
    stored === 'dark' ||
    stored === 'high-contrast' ||
    stored === 'system'
  ) {
    return stored;
  }
  return 'system';
}

export function setTheme(mode: ThemeMode) {
  localStorage.setItem(STORAGE_KEY, mode);
  applyClasses(resolveClasses(mode));
}

export function applyInitialTheme() {
  const mode = getStoredTheme();
  applyClasses(resolveClasses(mode));

  // Respond to OS theme changes when in 'system' mode. High-contrast
  // ignores the system preference â€” the user opted in explicitly.
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (getStoredTheme() === 'system') {
      applyClasses(resolveClasses('system'));
    }
  });
}
