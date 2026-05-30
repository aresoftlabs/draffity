<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Placeholder from '@tiptap/extension-placeholder';
import CharacterCount from '@tiptap/extension-character-count';
import Table from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import { computed, onBeforeUnmount, ref, watch, watchEffect } from 'vue';
import { useI18n } from 'vue-i18n';
import { Citation } from './extensions/citation';
import { CodexRef } from './extensions/codex-ref';
import { Footnote } from './extensions/footnote';
import { Image } from './extensions/image';
import { LinguisticFocus } from './extensions/linguistic-focus';
import { RepetitionHeatmap } from './extensions/repetition-heatmap';
import { ParagraphFade } from './extensions/paragraph-fade';
import { sanitizeUserCss, useEditorSettings } from '@/composables/useEditorSettings';
import { useMediaStore } from '@/stores/media';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    /** Canonical ProseMirror JSON state. When provided, takes precedence
     * over `modelValue` (HTML) for loading; useful for round-tripping
     * TipTap-only attributes (table column widths, etc.). */
    modelValueJson?: string | null;
    editable?: boolean;
    placeholder?: string;
    /** Text-column width in characters (composition mode, K-08). 0/undefined
     *  keeps the default 720px column. */
    paperWidthCh?: number;
  }>(),
  { editable: true, placeholder: '', modelValueJson: null, paperWidthCh: 0 },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:modelValueJson': [value: string];
}>();

const { t, locale } = useI18n();

/** Initial content prefers JSON when available; falls back to HTML. */
function initialContent(): string | object {
  if (props.modelValueJson) {
    try {
      return JSON.parse(props.modelValueJson) as object;
    } catch {
      // fall through to HTML
    }
  }
  return props.modelValue || '';
}

const editor = useEditor({
  content: initialContent(),
  editable: props.editable,
  extensions: [
    StarterKit.configure({
      // Use defaults; we add Underline separately since StarterKit omits it.
    }),
    Underline,
    CharacterCount,
    Placeholder.configure({
      placeholder: () => props.placeholder || t('editor.placeholder'),
    }),
    // Tables — header row enabled by default, columns are resizable by drag.
    Table.configure({ resizable: true, HTMLAttributes: { class: 'tiptap-table' } }),
    TableRow,
    TableHeader,
    TableCell,
    // Inline bibliographic citations. Pre-rendered label keeps export
    // trivial; the editor surface shows the label inline.
    Citation,
    // Inline codex cross-references `[[Name]]` resolved to an entry id.
    // Clicking dispatches `draffity:open-codex` on `window`.
    CodexRef,
    // Images stored as `<img data-media-id="…">` with a Vue NodeView that
    // resolves a Blob URL from the media store at render time.
    Image,
    // Footnotes — body lives inline as a node attribute; numbering at
    // export time. Click on a marker emits `draffity:open-footnote`.
    Footnote,
    // Linguistic Focus (J-06): toggleable highlight overlay (adverbs, passive
    // voice, dialogue) via decorations — never mutates the document.
    LinguisticFocus,
    // Repetition heatmap (J-08): local highlight of over-used words/phrases.
    RepetitionHeatmap,
    // Paragraph fade (K-08): dims non-focused blocks in composition mode.
    ParagraphFade,
  ],
  editorProps: {
    attributes: {
      class: 'tiptap-content prose-style focus:outline-none min-h-full px-10 py-8 leading-relaxed',
      spellcheck: 'true',
      // Hint the WebView's native spellcheck for the right dictionary.
      // Updated dynamically via the watcher below when the user changes UI
      // language.
      lang: locale.value,
    },
  },
  onUpdate: ({ editor: ed }) => {
    emit('update:modelValue', ed.getHTML());
    try {
      emit('update:modelValueJson', JSON.stringify(ed.getJSON()));
    } catch {
      // Highly unlikely (PM JSON is plain objects) — silently skip.
    }
  },
});

watch(locale, (next) => {
  // ProseMirror exposes the editable DOM element; toggling `lang` is enough
  // to swap the native spellchecker dictionary on the fly.
  editor.value?.view.dom.setAttribute('lang', next);
});

watch(
  () => [props.modelValueJson, props.modelValue],
  ([nextJson, nextHtml]) => {
    if (!editor.value) return;
    // Prefer JSON when set; only swap if it really differs from current state.
    if (nextJson) {
      try {
        const desired = JSON.parse(nextJson);
        const current = editor.value.getJSON();
        if (JSON.stringify(current) !== JSON.stringify(desired)) {
          editor.value.commands.setContent(desired, false);
        }
        return;
      } catch {
        // fall through to HTML
      }
    }
    if (editor.value.getHTML() !== (nextHtml ?? '')) {
      editor.value.commands.setContent(nextHtml ?? '', false);
    }
  },
);

watch(
  () => props.editable,
  (val) => {
    editor.value?.setEditable(val);
  },
);

const charCount = computed(() => {
  const ed = editor.value;
  if (!ed) return 0;
  // CharacterCount storage from extension.
  type Storage = { characters?: () => number };
  const storage = (ed.storage as Record<string, unknown>).characterCount as Storage | undefined;
  return storage?.characters?.() ?? 0;
});

const wordCount = computed(() => {
  const ed = editor.value;
  if (!ed) return 0;
  type Storage = { words?: () => number };
  const storage = (ed.storage as Record<string, unknown>).characterCount as Storage | undefined;
  return storage?.words?.() ?? 0;
});

const { customCss, fontFamily, customFontId } = useEditorSettings();
const mediaStore = useMediaStore();
const safeCustomCss = computed(() => sanitizeUserCss(customCss.value));

const customFontUrl = ref<string | null>(null);
const CUSTOM_FONT_NAME = 'DraffityCustomFont';
watch(
  customFontId,
  async (id) => {
    customFontUrl.value = null;
    if (!id) return;
    try {
      customFontUrl.value = await mediaStore.resolve(id);
    } catch {
      customFontUrl.value = null;
    }
  },
  { immediate: true },
);
const resolvedFontFamily = computed(() =>
  customFontUrl.value ? `'${CUSTOM_FONT_NAME}', ${fontFamily.value}` : fontFamily.value,
);

// Vue's template compiler ignores inline <style> tags (side-effect), so we
// inject the user's CSS into <head> as a singleton <style id="...">. The
// `watchEffect` keeps it in sync; `onBeforeUnmount` removes it if no other
// editor instance is mounted.
const STYLE_ID = 'draffity-editor-custom-css';
function getStyleEl(): HTMLStyleElement {
  let el = document.getElementById(STYLE_ID) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement('style');
    el.id = STYLE_ID;
    document.head.appendChild(el);
  }
  return el;
}
watchEffect(() => {
  if (typeof document === 'undefined') return;
  getStyleEl().textContent = safeCustomCss.value;
});

const FONT_STYLE_ID = 'draffity-editor-custom-font';
function getFontStyleEl(): HTMLStyleElement {
  let el = document.getElementById(FONT_STYLE_ID) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement('style');
    el.id = FONT_STYLE_ID;
    document.head.appendChild(el);
  }
  return el;
}
watchEffect(() => {
  if (typeof document === 'undefined') return;
  const url = customFontUrl.value;
  getFontStyleEl().textContent = url
    ? `@font-face { font-family: '${CUSTOM_FONT_NAME}'; src: url('${url}'); font-display: swap; }`
    : '';
});
onBeforeUnmount(() => {
  // Leave the style element in place if other instances may still need it;
  // if its content is empty there's nothing to remove either.
  const el = document.getElementById(STYLE_ID);
  if (el && !el.textContent) el.remove();
});

defineExpose({ editor, charCount, wordCount });
</script>

<template>
  <div
    class="tiptap-host h-full overflow-auto"
    :style="{
      '--editor-font-family': resolvedFontFamily,
      '--editor-max-width': paperWidthCh > 0 ? paperWidthCh + 'ch' : '720px',
    }"
  >
    <EditorContent :editor="editor" class="h-full" />
  </div>
</template>

<style scoped>
.tiptap-host :deep(.tiptap-content) {
  font-family: var(
    --editor-font-family,
    'Source Serif 4 Variable',
    'Source Serif 4',
    Georgia,
    'Times New Roman',
    serif
  );
  font-size: 18px;
  line-height: 1.7;
  max-width: var(--editor-max-width, 720px);
  margin: 0 auto;
}

/* Paragraph fade (K-08): dim non-focused blocks in composition mode. */
.tiptap-host :deep(.tiptap-content .pm-faded) {
  opacity: 0.32;
  transition: opacity 0.25s ease;
}

.tiptap-host :deep(.tiptap-content h1) {
  font-size: 2em;
  font-weight: 700;
  margin: 1.2em 0 0.6em;
}

.tiptap-host :deep(.tiptap-content h2) {
  font-size: 1.5em;
  font-weight: 700;
  margin: 1em 0 0.5em;
}

.tiptap-host :deep(.tiptap-content h3) {
  font-size: 1.2em;
  font-weight: 600;
  margin: 1em 0 0.5em;
}

.tiptap-host :deep(.tiptap-content p) {
  margin: 0 0 1em;
}

.tiptap-host :deep(.tiptap-content blockquote) {
  border-left: 3px solid var(--p-surface-300, #cbd5e1);
  padding-left: 1em;
  color: var(--p-surface-700, #475569);
  margin: 1em 0;
}

.tiptap-host :deep(.tiptap-content ul) {
  list-style: disc;
  padding-left: 1.4em;
  margin: 0 0 1em;
}

.tiptap-host :deep(.tiptap-content ol) {
  list-style: decimal;
  padding-left: 1.4em;
  margin: 0 0 1em;
}

.tiptap-host :deep(.tiptap-content hr) {
  border: 0;
  border-top: 1px solid var(--p-surface-300, #cbd5e1);
  margin: 2em 0;
}

.tiptap-host :deep(.tiptap-table) {
  border-collapse: collapse;
  table-layout: fixed;
  width: 100%;
  margin: 1em 0;
  overflow: hidden;
}
.tiptap-host :deep(.tiptap-table td),
.tiptap-host :deep(.tiptap-table th) {
  border: 1px solid var(--p-surface-300, #cbd5e1);
  padding: 0.4em 0.6em;
  vertical-align: top;
  min-width: 4em;
  position: relative;
}
.tiptap-host :deep(.tiptap-table th) {
  background: var(--p-surface-100, #f1f5f9);
  font-weight: 600;
  text-align: left;
}
/* TipTap's column resize handle */
.tiptap-host :deep(.tiptap-table .column-resize-handle) {
  position: absolute;
  right: -2px;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--p-primary-400, #38bdf8);
  pointer-events: none;
  opacity: 0;
}
.tiptap-host :deep(.tiptap-table:hover .column-resize-handle) {
  opacity: 0.5;
}
.tiptap-host :deep(.tableWrapper) {
  overflow-x: auto;
}
.tiptap-host :deep(.resize-cursor) {
  cursor: col-resize;
}

.tiptap-host :deep(.tiptap-content .citation) {
  display: inline;
  padding: 0 2px;
  border-radius: 3px;
  background: var(--p-primary-50, #eff6ff);
  color: var(--p-primary-700, #1d4ed8);
  font-size: 0.95em;
  cursor: default;
  white-space: nowrap;
}
.tiptap-host :deep(.tiptap-content .citation.ProseMirror-selectednode) {
  outline: 2px solid var(--p-primary-400, #60a5fa);
}

.tiptap-host :deep(.tiptap-content .codex-ref) {
  display: inline;
  padding: 0 2px;
  border-radius: 3px;
  background: var(--p-amber-50, #fffbeb);
  color: var(--p-amber-700, #b45309);
  font-size: 0.95em;
  cursor: pointer;
  white-space: nowrap;
  text-decoration: underline dotted;
  text-underline-offset: 2px;
}
.tiptap-host :deep(.tiptap-content .codex-ref:hover) {
  background: var(--p-amber-100, #fef3c7);
}
.tiptap-host :deep(.tiptap-content .codex-ref.ProseMirror-selectednode) {
  outline: 2px solid var(--p-amber-400, #fbbf24);
}

.tiptap-host :deep(.tiptap-content .footnote-ref) {
  display: inline-block;
  vertical-align: super;
  font-size: 0.7em;
  font-weight: 600;
  color: var(--p-primary-700, #1d4ed8);
  background: var(--p-primary-50, #eff6ff);
  border-radius: 2px;
  padding: 0 3px;
  margin: 0 1px;
  cursor: pointer;
  line-height: 1;
}
.tiptap-host :deep(.tiptap-content .footnote-ref:hover) {
  background: var(--p-primary-100, #dbeafe);
}
.tiptap-host :deep(.tiptap-content .footnote-ref.ProseMirror-selectednode) {
  outline: 2px solid var(--p-primary-400, #60a5fa);
}

.tiptap-host :deep(.tiptap-content code) {
  background: var(--p-surface-100, #f1f5f9);
  padding: 0.1em 0.3em;
  border-radius: 3px;
  font-family: 'JetBrains Mono', Menlo, Consolas, monospace;
  font-size: 0.92em;
}

/* Placeholder */
.tiptap-host :deep(.tiptap-content p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  color: var(--p-surface-400, #94a3b8);
  pointer-events: none;
  height: 0;
  float: left;
}
</style>
