/**
 * Reads / writes integer-minute timer settings from `localStorage` under
 * the `draffity.timer.*` namespace. Tolerant: missing keys, non-numeric
 * values and zero/negative numbers all fall back to the caller's default.
 */
const STORAGE_PREFIX = 'draffity.timer.';

export function useTimerStorage() {
  function load(key: string, fallback: number): number {
    if (typeof localStorage === 'undefined') return fallback;
    const raw = localStorage.getItem(STORAGE_PREFIX + key);
    if (!raw) return fallback;
    const n = Number(raw);
    return Number.isFinite(n) && n > 0 ? n : fallback;
  }

  function save(key: string, value: number) {
    if (typeof localStorage === 'undefined') return;
    localStorage.setItem(STORAGE_PREFIX + key, String(value));
  }

  return { load, save };
}
