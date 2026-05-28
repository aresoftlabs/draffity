<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import type { Editor } from '@tiptap/vue-3';
import { useFindReplace } from '@/composables/useFindReplace';

const props = defineProps<{
  visible: boolean;
  editor: Editor | null;
  mode: 'find' | 'replace';
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  'update:visible': [v: boolean];
}>();

const { t } = useI18n();
const editorRef = computed(() => props.editor);
const fr = useFindReplace(editorRef);

const findInput = ref<InstanceType<typeof InputText> | null>(null);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      // If text is selected when opening, prefill query with it.
      const sel = props.editor?.state.selection;
      if (sel && !sel.empty) {
        const slice = props.editor!.state.doc.textBetween(sel.from, sel.to, ' ');
        if (slice && slice.length < 80) fr.query.value = slice;
      }
      void nextTick(focusFind);
    } else {
      fr.reset();
    }
  },
);

function focusFind() {
  const inst = findInput.value as unknown as { $el?: HTMLInputElement };
  inst?.$el?.focus();
  inst?.$el?.select();
}

function close() {
  emit('update:visible', false);
  // Return focus to the editor so the user can keep typing.
  props.editor?.commands.focus();
}

function onFindKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    close();
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (e.shiftKey) fr.prev();
    else fr.next();
  }
}

const counter = computed(() => {
  if (!fr.query.value) return '';
  if (fr.totalMatches.value === 0) return t('find.noMatches');
  return t('find.counter', {
    current: fr.currentIndex.value + 1,
    total: fr.totalMatches.value,
  });
});
</script>

<template>
  <div
    v-if="visible"
    class="flex items-center gap-2 px-3 py-2 border-b border-surface-200 dark:border-surface-700 bg-surface-50 dark:bg-surface-900"
    role="search"
  >
    <i class="pi pi-search opacity-60 text-sm" aria-hidden="true" />
    <InputText
      ref="findInput"
      v-model="fr.query.value"
      :placeholder="t('find.findPlaceholder')"
      :aria-label="t('find.findPlaceholder')"
      class="!py-1 !text-sm flex-1 max-w-md"
      @keydown="onFindKey"
    />
    <span
      class="text-xs opacity-60 min-w-[5rem]"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      >{{ counter }}</span
    >

    <Button
      v-tooltip.bottom="t('find.previous')"
      icon="pi pi-chevron-up"
      text
      severity="secondary"
      size="small"
      :disabled="!fr.hasMatches.value"
      :aria-label="t('find.previous')"
      @click="fr.prev"
    />
    <Button
      v-tooltip.bottom="t('find.next')"
      icon="pi pi-chevron-down"
      text
      severity="secondary"
      size="small"
      :disabled="!fr.hasMatches.value"
      :aria-label="t('find.next')"
      @click="fr.next"
    />

    <template v-if="mode === 'replace' && !readOnly">
      <span class="w-px h-5 bg-surface-300 dark:bg-surface-700 mx-1" aria-hidden="true" />
      <InputText
        v-model="fr.replacement.value"
        :placeholder="t('find.replacePlaceholder')"
        :aria-label="t('find.replacePlaceholder')"
        class="!py-1 !text-sm max-w-xs"
        @keydown.escape.prevent="close"
      />
      <Button
        :label="t('find.replace')"
        text
        severity="secondary"
        size="small"
        :disabled="!fr.hasMatches.value"
        @click="fr.replaceCurrent"
      />
      <Button
        :label="t('find.replaceAll')"
        text
        size="small"
        :disabled="!fr.hasMatches.value"
        @click="fr.replaceAll"
      />
    </template>

    <span class="flex-1" aria-hidden="true" />
    <Button
      v-tooltip.bottom="t('find.close')"
      icon="pi pi-times"
      text
      severity="secondary"
      size="small"
      :aria-label="t('find.close')"
      @click="close"
    />
  </div>
</template>
