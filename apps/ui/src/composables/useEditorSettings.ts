import { ref, watch, type Ref } from 'vue';
import { ipc } from '@/services/ipc';

export type EditorFont = 'serif' | 'sans' | 'mono';

const KEY_AUTOSAVE = 'editor.autosave_ms';
const KEY_FONT = 'editor.font';
const KEY_CUSTOM_CSS = 'editor.custom_css';

const DEFAULT_AUTOSAVE = 500;
const DEFAULT_FONT: EditorFont = 'serif';
const MAX_CUSTOM_CSS = 4096;

const autosaveMs = ref(DEFAULT_AUTOSAVE);
const font = ref<EditorFont>(DEFAULT_FONT);
const customCss = ref('');
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
  customCss: Ref<string>;
  loaded: Ref<boolean>;
  reload: () => Promise<void>;
} {
  if (!inFlight) {
    inFlight = load();
  }

  return {
    autosaveMs,
    font,
    customCss,
    loaded,
    reload: () => {
      inFlight = load();
      return inFlight;
    },
  };
}

async function load() {
  try {
    const [auto, f, css] = await Promise.all([
      ipc.getSetting(KEY_AUTOSAVE),
      ipc.getSetting(KEY_FONT),
      ipc.getSetting(KEY_CUSTOM_CSS),
    ]);
    if (auto) {
      const n = Number.parseInt(auto, 10);
      if (Number.isFinite(n) && n >= 100) autosaveMs.value = n;
    }
    if (f === 'serif' || f === 'sans' || f === 'mono') {
      font.value = f;
    }
    if (typeof css === 'string') {
      customCss.value = css.slice(0, MAX_CUSTOM_CSS);
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

watch(customCss, async (v) => {
  if (!loaded.value) return;
  // Defence: cap length so a runaway paste can't push the settings row past
  // a reasonable size. 4 KB is more than enough for the customisation knobs
  // a writer would want.
  if (v.length > MAX_CUSTOM_CSS) {
    customCss.value = v.slice(0, MAX_CUSTOM_CSS);
    return;
  }
  try {
    await ipc.setSetting(KEY_CUSTOM_CSS, v);
  } catch {
    /* ignore */
  }
});

/** Sanitises user CSS for injection. Strips `</style>` to prevent breaking
 *  out of the `<style>` block and `@import`/`url()` to keep the editor from
 *  fetching arbitrary network resources from a writer's CSS snippet. */
export function sanitizeUserCss(input: string): string {
  return input
    .replace(/<\/style>/gi, '')
    .replace(/@import\s+[^;]+;?/gi, '')
    .replace(/url\s*\(/gi, '/*url(*/');
}
