import { ref, watch, type Ref } from 'vue';
import { ipc } from '@/services/ipc';

/** Built-in presets — kept for back-compat with the previous SelectButton. */
export type EditorFont = 'serif' | 'sans' | 'mono';

/** CSS font-family stack mapped from a built-in preset. Kept here (and not
 *  in the Settings view) so the editor can resolve the family from a raw
 *  setting value without round-tripping through Vue components. */
const BUILT_IN_FAMILIES: Record<EditorFont, string> = {
  serif: "'Source Serif 4 Variable', 'Source Serif 4', Lora, Georgia, 'Times New Roman', serif",
  sans: "'Inter Variable', 'Inter', system-ui, sans-serif",
  mono: "'JetBrains Mono', Menlo, Consolas, monospace",
};

const KEY_AUTOSAVE = 'editor.autosave_ms';
const KEY_FONT = 'editor.font';
const KEY_FONT_FAMILY = 'editor.font_family';
const KEY_FONT_CUSTOM_ID = 'editor.font_custom_id';
const KEY_CUSTOM_CSS = 'editor.custom_css';
const KEY_PAPER_WIDTH = 'editor.paper_width_ch';
const KEY_COMPOSITION_BG = 'editor.composition_bg';
const KEY_FADE_LEVEL = 'editor.fade_level';

const DEFAULT_AUTOSAVE = 500;
const DEFAULT_FONT: EditorFont = 'serif';
const MAX_CUSTOM_CSS = 4096;
const DEFAULT_PAPER_WIDTH = 80;

/** Composition-mode (K-08) fade level: dim everything but the focused unit. */
export type FadeLevel = 'none' | 'paragraph';

const autosaveMs = ref(DEFAULT_AUTOSAVE);
const font = ref<EditorFont>(DEFAULT_FONT);
const fontFamily = ref(BUILT_IN_FAMILIES.serif);
const customFontId = ref<string | null>(null);
const customCss = ref('');
/** Paper width in characters for composition mode (50–140); 0 = full width. */
const paperWidthCh = ref<number>(DEFAULT_PAPER_WIDTH);
/** Composition background — a CSS color string; '' = theme default. */
const compositionBg = ref<string>('');
const fadeLevel = ref<FadeLevel>('none');
const loaded = ref(false);

let inFlight: Promise<void> | null = null;

/**
 * Reactive accessors for editor settings backed by the Rust `settings` table.
 * Loads on first call (or whenever `reload` is invoked) and writes back on
 * change. Safe to call from multiple components — state is module-shared.
 *
 * `fontFamily` is the resolved CSS font-family stack the editor applies;
 * `customFontId` (when non-null) points at a media row whose bytes the
 * editor side-loads via `@font-face` so uploaded fonts work offline.
 */
export function useEditorSettings(): {
  autosaveMs: Ref<number>;
  font: Ref<EditorFont>;
  fontFamily: Ref<string>;
  customFontId: Ref<string | null>;
  customCss: Ref<string>;
  paperWidthCh: Ref<number>;
  compositionBg: Ref<string>;
  fadeLevel: Ref<FadeLevel>;
  loaded: Ref<boolean>;
  reload: () => Promise<void>;
} {
  if (!inFlight) {
    inFlight = load();
  }

  return {
    autosaveMs,
    font,
    fontFamily,
    customFontId,
    customCss,
    paperWidthCh,
    compositionBg,
    fadeLevel,
    loaded,
    reload: () => {
      inFlight = load();
      return inFlight;
    },
  };
}

async function load() {
  try {
    const [auto, f, fam, customId, css, paper, bg, fade] = await Promise.all([
      ipc.getSetting(KEY_AUTOSAVE),
      ipc.getSetting(KEY_FONT),
      ipc.getSetting(KEY_FONT_FAMILY),
      ipc.getSetting(KEY_FONT_CUSTOM_ID),
      ipc.getSetting(KEY_CUSTOM_CSS),
      ipc.getSetting(KEY_PAPER_WIDTH),
      ipc.getSetting(KEY_COMPOSITION_BG),
      ipc.getSetting(KEY_FADE_LEVEL),
    ]);
    if (auto) {
      const n = Number.parseInt(auto, 10);
      if (Number.isFinite(n) && n >= 100) autosaveMs.value = n;
    }
    if (f === 'serif' || f === 'sans' || f === 'mono') {
      font.value = f;
    }
    if (typeof fam === 'string' && fam.trim().length > 0) {
      fontFamily.value = fam;
    } else {
      // Migrate from the legacy preset-only setting so users who set their
      // font under v0.9 keep it after the upgrade.
      fontFamily.value = BUILT_IN_FAMILIES[font.value];
    }
    customFontId.value = typeof customId === 'string' && customId.length > 0 ? customId : null;
    if (typeof css === 'string') {
      customCss.value = css.slice(0, MAX_CUSTOM_CSS);
    }
    if (paper) {
      const n = Number.parseInt(paper, 10);
      if (Number.isFinite(n) && (n === 0 || (n >= 50 && n <= 140))) paperWidthCh.value = n;
    }
    if (typeof bg === 'string') compositionBg.value = bg;
    if (fade === 'none' || fade === 'paragraph') fadeLevel.value = fade;
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

watch(fontFamily, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_FONT_FAMILY, v);
  } catch {
    /* ignore */
  }
});

watch(customFontId, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_FONT_CUSTOM_ID, v ?? '');
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

watch(paperWidthCh, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_PAPER_WIDTH, String(v));
  } catch {
    /* ignore */
  }
});

watch(compositionBg, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_COMPOSITION_BG, v);
  } catch {
    /* ignore */
  }
});

watch(fadeLevel, async (v) => {
  if (!loaded.value) return;
  try {
    await ipc.setSetting(KEY_FADE_LEVEL, v);
  } catch {
    /* ignore */
  }
});

/** Map a built-in preset to its CSS font-family stack. Used by the
 *  Settings UI to resolve the dropdown choice into the stored family. */
export function builtInFamily(preset: EditorFont): string {
  return BUILT_IN_FAMILIES[preset];
}

/** Sanitises user CSS for injection. Strips `</style>` to prevent breaking
 *  out of the `<style>` block and `@import`/`url()` to keep the editor from
 *  fetching arbitrary network resources from a writer's CSS snippet. */
export function sanitizeUserCss(input: string): string {
  return input
    .replace(/<\/style>/gi, '')
    .replace(/@import\s+[^;]+;?/gi, '')
    .replace(/url\s*\(/gi, '/*url(*/');
}
