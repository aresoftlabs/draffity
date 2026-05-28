import { ref, watch, type Ref } from 'vue';
import { ipc } from '@/services/ipc';

export type EditorFont = 'serif' | 'sans' | 'mono';

const KEY_AUTOSAVE = 'editor.autosave_ms';
const KEY_FONT = 'editor.font';

const DEFAULT_AUTOSAVE = 500;
const DEFAULT_FONT: EditorFont = 'serif';

const autosaveMs = ref(DEFAULT_AUTOSAVE);
const font = ref<EditorFont>(DEFAULT_FONT);
const loaded = ref(false);

let inFlight: Promise<void> | null = null;

/**
 * Reactive accessors for editor settings backed by the Rust `settings` table.
 * Loads on first call (or whenever `reload` is invoked) and writes back on
 * change. Safe to call from multiple components — state is module-shared.
 */
export function useEditorSettings(): {
  autosaveMs: Ref<number>;
  font: Ref<EditorFont>;
  loaded: Ref<boolean>;
  reload: () => Promise<void>;
} {
  if (!inFlight) {
    inFlight = load();
  }

  return {
    autosaveMs,
    font,
    loaded,
    reload: () => {
      inFlight = load();
      return inFlight;
    },
  };
}

async function load() {
  try {
    const [auto, f] = await Promise.all([ipc.getSetting(KEY_AUTOSAVE), ipc.getSetting(KEY_FONT)]);
    if (auto) {
      const n = Number.parseInt(auto, 10);
      if (Number.isFinite(n) && n >= 100) autosaveMs.value = n;
    }
    if (f === 'serif' || f === 'sans' || f === 'mono') {
      font.value = f;
    }
  } finally {
    loaded.value = true;
  }
}

watch(autosaveMs, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_AUTOSAVE, String(v));
  } catch {
    /* ignore — toast surface in callers */
  }
});

watch(font, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_FONT, v);
  } catch {
    /* ignore */
  }
});
