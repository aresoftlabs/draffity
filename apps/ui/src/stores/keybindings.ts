import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { ipc } from '@/services/ipc';

/** Stable action identifiers wired to handlers in views. Keep this list in
 *  sync with `useShortcuts` callers so the Settings UI can render every
 *  rebindable command. */
export const SHORTCUT_ACTIONS = [
  'flushSave',
  'newChapter',
  'searchProject',
  'findInDocument',
  'replaceInDocument',
  'prevDocument',
  'nextDocument',
  'focusMode',
  'commandPalette',
] as const;

export type ShortcutAction = (typeof SHORTCUT_ACTIONS)[number];

/** Default combo per action. Modifier order is `ctrl, shift, alt, key`. */
export const DEFAULT_BINDINGS: Record<ShortcutAction, string> = {
  flushSave: 'ctrl+s',
  newChapter: 'ctrl+n',
  searchProject: 'ctrl+shift+f',
  findInDocument: 'ctrl+f',
  replaceInDocument: 'ctrl+h',
  prevDocument: 'ctrl+,',
  nextDocument: 'ctrl+.',
  focusMode: 'f11',
  commandPalette: 'ctrl+k',
};

const SETTING_KEY = 'editor.keybindings';

export const useKeybindingsStore = defineStore('keybindings', () => {
  const bindings = ref<Record<ShortcutAction, string>>({ ...DEFAULT_BINDINGS });
  const loaded = ref(false);

  /** Loads persisted bindings from the backend `settings` table. Unknown
   *  actions in storage are dropped silently — the user might be on an
   *  older binding payload after upgrading. */
  async function load() {
    if (loaded.value) return;
    try {
      const raw = await ipc.getSetting(SETTING_KEY);
      if (raw) {
        const parsed = JSON.parse(raw) as Partial<Record<string, string>>;
        for (const action of SHORTCUT_ACTIONS) {
          const value = parsed[action];
          if (typeof value === 'string' && value.trim()) {
            bindings.value[action] = normalizeCombo(value);
          }
        }
      }
    } finally {
      loaded.value = true;
    }
  }

  async function set(action: ShortcutAction, combo: string) {
    bindings.value[action] = normalizeCombo(combo);
    await persist();
  }

  async function reset(action: ShortcutAction) {
    bindings.value[action] = DEFAULT_BINDINGS[action];
    await persist();
  }

  async function persist() {
    await ipc.setSetting(SETTING_KEY, JSON.stringify(bindings.value));
  }

  /** Reverse map combo → action. Used by `useShortcuts` to dispatch a
   *  KeyboardEvent without scanning the whole bindings dict per key press. */
  const comboMap = computed(() => {
    const m = new Map<string, ShortcutAction>();
    for (const action of SHORTCUT_ACTIONS) {
      m.set(bindings.value[action], action);
    }
    return m;
  });

  return { bindings, loaded, load, set, reset, comboMap };
});

/** Format the active modifier+key combo as a normalized key, e.g. `ctrl+s`.
 *  Exported so the Settings UI can render the captured key the same way the
 *  shortcut dispatcher does. */
export function formatCombo(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('ctrl');
  if (e.shiftKey) parts.push('shift');
  if (e.altKey) parts.push('alt');
  const key = e.key.toLowerCase();
  // Filter modifier-only events so the capture UI doesn't accept "ctrl" alone.
  if (key === 'control' || key === 'meta' || key === 'shift' || key === 'alt') {
    return parts.join('+');
  }
  parts.push(key);
  return parts.join('+');
}

function normalizeCombo(raw: string): string {
  return raw.trim().toLowerCase();
}
