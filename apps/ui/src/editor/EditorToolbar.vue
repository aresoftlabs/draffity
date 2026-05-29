<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Editor } from '@tiptap/vue-3';
import Button from 'primevue/button';

const props = defineProps<{
  editor: Editor | null;
  disabled?: boolean;
  /** Linguistic Focus highlight overlay active state (J-06). */
  linguisticFocusActive?: boolean;
}>();

const emit = defineEmits<{
  'open-citation-picker': [];
  'open-codex-picker': [];
  'insert-image': [];
  'insert-footnote': [];
  'toggle-linguistic-focus': [];
}>();

const { t } = useI18n();

const isReady = computed(() => !!props.editor && !props.disabled);

function isActive(name: string, attrs?: Record<string, unknown>) {
  return !!props.editor && props.editor.isActive(name, attrs);
}

// Table commands operate on cells; their availability depends on whether
// the cursor sits inside a table node.
const isInTable = computed(() => isActive('table'));
</script>

<template>
  <div
    role="toolbar"
    :aria-label="t('toolbar.label')"
    class="flex items-center gap-1 flex-wrap px-3 py-2 border-b border-surface-200 dark:border-surface-700 bg-surface-50/80 dark:bg-surface-900/80 backdrop-blur"
  >
    <Button
      v-tooltip.bottom="t('toolbar.heading1')"
      :aria-label="t('toolbar.heading1')"
      :aria-pressed="isActive('heading', { level: 1 })"
      icon="pi pi-hashtag"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('heading', { level: 1 }) }"
      @click="() => editor?.chain().focus().toggleHeading({ level: 1 }).run()"
    >
      <span class="text-xs font-bold" aria-hidden="true">H1</span>
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.heading2')"
      :aria-label="t('toolbar.heading2')"
      :aria-pressed="isActive('heading', { level: 2 })"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('heading', { level: 2 }) }"
      @click="() => editor?.chain().focus().toggleHeading({ level: 2 }).run()"
    >
      <span class="text-xs font-bold" aria-hidden="true">H2</span>
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.heading3')"
      :aria-label="t('toolbar.heading3')"
      :aria-pressed="isActive('heading', { level: 3 })"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('heading', { level: 3 }) }"
      @click="() => editor?.chain().focus().toggleHeading({ level: 3 }).run()"
    >
      <span class="text-xs font-bold" aria-hidden="true">H3</span>
    </Button>

    <span class="w-px h-6 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.bold')"
      :aria-label="t('toolbar.bold')"
      :aria-pressed="isActive('bold')"
      icon="pi pi-bold"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('bold') }"
      @click="() => editor?.chain().focus().toggleBold().run()"
    />
    <Button
      v-tooltip.bottom="t('toolbar.italic')"
      :aria-label="t('toolbar.italic')"
      :aria-pressed="isActive('italic')"
      icon="pi pi-italic"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('italic') }"
      @click="() => editor?.chain().focus().toggleItalic().run()"
    />
    <Button
      v-tooltip.bottom="t('toolbar.underline')"
      :aria-label="t('toolbar.underline')"
      :aria-pressed="isActive('underline')"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('underline') }"
      @click="() => editor?.chain().focus().toggleUnderline().run()"
    >
      <span class="underline text-xs font-semibold" aria-hidden="true">U</span>
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.strike')"
      :aria-label="t('toolbar.strike')"
      :aria-pressed="isActive('strike')"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('strike') }"
      @click="() => editor?.chain().focus().toggleStrike().run()"
    >
      <span class="line-through text-xs font-semibold" aria-hidden="true">S</span>
    </Button>

    <span class="w-px h-6 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.bulletList')"
      :aria-label="t('toolbar.bulletList')"
      :aria-pressed="isActive('bulletList')"
      icon="pi pi-list"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('bulletList') }"
      @click="() => editor?.chain().focus().toggleBulletList().run()"
    />
    <Button
      v-tooltip.bottom="t('toolbar.orderedList')"
      :aria-label="t('toolbar.orderedList')"
      :aria-pressed="isActive('orderedList')"
      icon="pi pi-sort-numeric-down"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('orderedList') }"
      @click="() => editor?.chain().focus().toggleOrderedList().run()"
    />
    <Button
      v-tooltip.bottom="t('toolbar.blockquote')"
      :aria-label="t('toolbar.blockquote')"
      :aria-pressed="isActive('blockquote')"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': isActive('blockquote') }"
      @click="() => editor?.chain().focus().toggleBlockquote().run()"
    >
      <i class="pi pi-comment text-xs" aria-hidden="true" />
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.horizontalRule')"
      :aria-label="t('toolbar.horizontalRule')"
      icon="pi pi-minus"
      text
      severity="secondary"
      :disabled="!isReady"
      @click="() => editor?.chain().focus().setHorizontalRule().run()"
    />

    <span class="w-px h-6 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.insertTable')"
      :aria-label="t('toolbar.insertTable')"
      icon="pi pi-table"
      text
      severity="secondary"
      :disabled="!isReady || isInTable"
      @click="
        () => editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
      "
    />
    <Button
      v-tooltip.bottom="t('toolbar.tableAddRow')"
      :aria-label="t('toolbar.tableAddRow')"
      icon="pi pi-plus"
      text
      severity="secondary"
      :disabled="!isReady || !isInTable"
      @click="() => editor?.chain().focus().addRowAfter().run()"
    >
      <span class="text-[10px] font-semibold ml-1" aria-hidden="true">R</span>
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.tableAddCol')"
      :aria-label="t('toolbar.tableAddCol')"
      icon="pi pi-plus"
      text
      severity="secondary"
      :disabled="!isReady || !isInTable"
      @click="() => editor?.chain().focus().addColumnAfter().run()"
    >
      <span class="text-[10px] font-semibold ml-1" aria-hidden="true">C</span>
    </Button>
    <Button
      v-tooltip.bottom="t('toolbar.tableDelete')"
      :aria-label="t('toolbar.tableDelete')"
      icon="pi pi-trash"
      text
      severity="secondary"
      :disabled="!isReady || !isInTable"
      @click="() => editor?.chain().focus().deleteTable().run()"
    />

    <span class="w-px h-6 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.insertCitation')"
      :aria-label="t('toolbar.insertCitation')"
      icon="pi pi-at"
      text
      severity="secondary"
      :disabled="!isReady"
      @click="emit('open-citation-picker')"
    />
    <Button
      v-tooltip.bottom="t('toolbar.insertCodexRef')"
      :aria-label="t('toolbar.insertCodexRef')"
      icon="pi pi-link"
      text
      severity="secondary"
      :disabled="!isReady"
      @click="emit('open-codex-picker')"
    />
    <Button
      v-tooltip.bottom="t('toolbar.insertImage')"
      :aria-label="t('toolbar.insertImage')"
      icon="pi pi-image"
      text
      severity="secondary"
      :disabled="!isReady"
      @click="emit('insert-image')"
    />
    <Button
      v-tooltip.bottom="t('toolbar.insertFootnote')"
      :aria-label="t('toolbar.insertFootnote')"
      icon="pi pi-asterisk"
      text
      severity="secondary"
      :disabled="!isReady"
      @click="emit('insert-footnote')"
    />

    <span class="w-px h-6 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.linguisticFocus')"
      :aria-label="t('toolbar.linguisticFocus')"
      :aria-pressed="linguisticFocusActive"
      icon="pi pi-eye"
      text
      severity="secondary"
      :disabled="!isReady"
      :class="{ 'p-button-outlined': linguisticFocusActive }"
      @click="emit('toggle-linguistic-focus')"
    />

    <span class="flex-1" aria-hidden="true" />

    <Button
      v-tooltip.bottom="t('toolbar.undo')"
      :aria-label="t('toolbar.undo')"
      icon="pi pi-undo"
      text
      severity="secondary"
      :disabled="!isReady || !editor?.can().undo()"
      @click="() => editor?.chain().focus().undo().run()"
    />
    <Button
      v-tooltip.bottom="t('toolbar.redo')"
      :aria-label="t('toolbar.redo')"
      icon="pi pi-refresh"
      text
      severity="secondary"
      :disabled="!isReady || !editor?.can().redo()"
      @click="() => editor?.chain().focus().redo().run()"
    />
  </div>
</template>
