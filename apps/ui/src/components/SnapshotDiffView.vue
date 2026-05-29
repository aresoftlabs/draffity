<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { htmlToLines, lineDiff, type DiffOp } from '@/composables/useTextDiff';

/**
 * Side-by-side snapshot diff. Compares two HTML payloads by stripping
 * down to paragraph text (so inline marks like bold/italic don't fire
 * spurious changes) and running an LCS line diff. The two columns stay
 * aligned: removed-only rows leave a blank cell on the right and vice
 * versa, so the reader's eye can track which paragraph went where.
 */

const props = defineProps<{
  visible: boolean;
  beforeLabel?: string;
  afterLabel?: string;
  beforeHtml: string;
  afterHtml: string;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

const { t } = useI18n();

const ops = computed<DiffOp[]>(() => {
  if (!props.visible) return [];
  return lineDiff(htmlToLines(props.beforeHtml), htmlToLines(props.afterHtml));
});

const stats = computed(() => {
  let add = 0;
  let remove = 0;
  for (const op of ops.value) {
    if (op.kind === 'add') add++;
    else if (op.kind === 'remove') remove++;
  }
  return { add, remove };
});

function close() {
  emit('update:visible', false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="t('diff.title')"
    modal
    :style="{ width: '90vw', maxWidth: '1200px' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex items-center justify-between gap-3 text-xs opacity-70 mb-3">
      <div class="flex gap-3">
        <span>
          <span class="font-semibold">{{ beforeLabel ?? t('diff.before') }}</span>
        </span>
        <span class="text-amber-700 dark:text-amber-300">{{ `−${stats.remove}` }}</span>
        <span class="text-emerald-700 dark:text-emerald-300">{{ `+${stats.add}` }}</span>
      </div>
      <span class="font-semibold">{{ afterLabel ?? t('diff.after') }}</span>
    </div>

    <div
      v-if="ops.length === 0"
      class="text-sm opacity-60 p-6 text-center border border-dashed border-surface-300 dark:border-surface-700 rounded"
    >
      {{ t('diff.identical') }}
    </div>

    <div
      v-else
      class="grid grid-cols-2 gap-2 font-mono text-sm max-h-[60vh] overflow-auto border border-surface-200 dark:border-surface-700 rounded"
    >
      <template v-for="(op, i) in ops" :key="i">
        <div
          :class="[
            'px-3 py-1 whitespace-pre-wrap break-words',
            op.kind === 'remove'
              ? 'bg-rose-50 dark:bg-rose-900/20 text-rose-900 dark:text-rose-200'
              : op.kind === 'add'
                ? 'opacity-40'
                : '',
          ]"
        >
          {{ op.before ?? '' }}
        </div>
        <div
          :class="[
            'px-3 py-1 whitespace-pre-wrap break-words',
            op.kind === 'add'
              ? 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-900 dark:text-emerald-200'
              : op.kind === 'remove'
                ? 'opacity-40'
                : '',
          ]"
        >
          {{ op.after ?? '' }}
        </div>
      </template>
    </div>

    <template #footer>
      <Button :label="t('diff.close')" @click="close" />
    </template>
  </Dialog>
</template>
