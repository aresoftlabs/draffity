import { onBeforeUnmount, onMounted } from 'vue';
import { formatCombo, useKeybindingsStore, type ShortcutAction } from '@/stores/keybindings';

export type ShortcutHandlers = Partial<Record<ShortcutAction, (e: KeyboardEvent) => void>>;

/**
 * Register keyboard shortcuts during the lifetime of the calling component.
 * Handlers are keyed by stable **action** ids (`flushSave`, `newChapter`,
 * …); the actual combo comes from the keybindings store so users can
 * rebind any action from Settings.
 */
export function useShortcuts(handlers: ShortcutHandlers) {
  const store = useKeybindingsStore();
  // Kick off the load lazily — every consumer benefits, no harm in racing
  // (the dispatcher just sees defaults for the first frame).
  void store.load();

  function onKey(e: KeyboardEvent) {
    const combo = formatCombo(e);
    const action = store.comboMap.get(combo);
    if (!action) return;
    const handler = handlers[action];
    if (!handler) return;
    e.preventDefault();
    handler(e);
  }
  onMounted(() => window.addEventListener('keydown', onKey));
  onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
}

export { formatCombo as _formatCombo };
