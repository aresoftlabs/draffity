import { onBeforeUnmount, onMounted } from 'vue';

export interface ShortcutMap {
  [combo: string]: (e: KeyboardEvent) => void;
}

/** Format the active modifier+key combo as a normalized key, e.g. `ctrl+s`. */
function formatCombo(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('ctrl');
  if (e.shiftKey) parts.push('shift');
  if (e.altKey) parts.push('alt');
  parts.push(e.key.toLowerCase());
  return parts.join('+');
}

/**
 * Register keyboard shortcuts during the lifetime of the calling component.
 * Combos are normalized as `ctrl+s`, `ctrl+shift+n`, etc. `meta` is mapped to
 * `ctrl` so Mac keyboard support is automatic.
 */
export function useShortcuts(map: ShortcutMap) {
  function onKey(e: KeyboardEvent) {
    const combo = formatCombo(e);
    const handler = map[combo];
    if (handler) {
      e.preventDefault();
      handler(e);
    }
  }
  onMounted(() => window.addEventListener('keydown', onKey));
  onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
}

export { formatCombo as _formatCombo };
