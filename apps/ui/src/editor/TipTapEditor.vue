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
import { computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Citation } from './extensions/citation';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    /** Canonical ProseMirror JSON state. When provided, takes precedence
     * over `modelValue` (HTML) for loading; useful for round-tripping
     * TipTap-only attributes (table column widths, etc.). */
    modelValueJson?: string | null;
    editable?: boolean;
    placeholder?: string;
  }>(),
  { editable: true, placeholder: '', modelValueJson: null },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:modelValueJson': [value: string];
}>();

const { locale } = useI18n();

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
      placeholder: () => props.placeholder || 'Empieza a escribir…',
    }),
    // Tables — header row enabled by default, columns are resizable by drag.
    Table.configure({ resizable: true, HTMLAttributes: { class: 'tiptap-table' } }),
    TableRow,
    TableHeader,
    TableCell,
    // Inline bibliographic citations. Pre-rendered label keeps export
    // trivial; the editor surface shows the label inline.
    Citation,
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

defineExpose({ editor, charCount, wordCount });
</script>

<template>
  <div class="tiptap-host h-full overflow-auto">
    <EditorContent :editor="editor" class="h-full" />
  </div>
</template>

<style scoped>
.tiptap-host :deep(.tiptap-content) {
  font-family: Lora, Georgia, 'Times New Roman', serif;
  font-size: 18px;
  line-height: 1.7;
  max-width: 720px;
  margin: 0 auto;
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
