<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Placeholder from '@tiptap/extension-placeholder';
import CharacterCount from '@tiptap/extension-character-count';
import { computed, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    editable?: boolean;
    placeholder?: string;
  }>(),
  { editable: true, placeholder: '' },
);

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const editor = useEditor({
  content: props.modelValue || '',
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
  ],
  editorProps: {
    attributes: {
      class: 'tiptap-content prose-style focus:outline-none min-h-full px-10 py-8 leading-relaxed',
      spellcheck: 'true',
    },
  },
  onUpdate: ({ editor: ed }) => {
    emit('update:modelValue', ed.getHTML());
  },
});

watch(
  () => props.modelValue,
  (val) => {
    if (!editor.value) return;
    if (editor.value.getHTML() !== val) {
      editor.value.commands.setContent(val || '', false);
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
