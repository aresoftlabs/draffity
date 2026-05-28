<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import type { Citation } from '@draffity/shared-types';
import { useCitationsStore } from '@/stores/citations';

const props = defineProps<{
  visible: boolean;
  projectId: string | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  /** Fired when a citation is picked. Carries the pre-rendered label so the
   *  editor can insert the node without re-deriving it. */
  pick: [payload: { key: string; label: string }];
}>();

const { t } = useI18n();
const store = useCitationsStore();

const query = ref('');

watch(
  () => [props.visible, props.projectId] as const,
  async ([v, pid]) => {
    if (!v || !pid) return;
    query.value = '';
    await store.loadFor(pid);
  },
);

const filtered = computed<Citation[]>(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return store.list;
  return store.list.filter((c) => {
    if (c.key.toLowerCase().includes(q)) return true;
    const author = (c.fields.author ?? '').toLowerCase();
    const title = (c.fields.title ?? '').toLowerCase();
    return author.includes(q) || title.includes(q);
  });
});

function close() {
  emit('update:visible', false);
}

function onPick(c: Citation) {
  emit('pick', { key: c.key, label: store.labelFor(c) });
  close();
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('bibliography.pickerTitle')"
    :style="{ width: '32rem', maxHeight: '80vh' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-3">
      <p class="text-sm opacity-70">{{ t('bibliography.pickerSubtitle') }}</p>

      <InputText
        v-model="query"
        :placeholder="t('bibliography.pickerSearch')"
        class="w-full"
        autofocus
      />

      <div
        class="flex flex-col gap-1 max-h-[50vh] overflow-y-auto rounded border border-surface-200 dark:border-surface-700"
      >
        <button
          v-for="c in filtered"
          :key="c.id"
          type="button"
          class="text-left p-2 hover:bg-surface-100 dark:hover:bg-surface-800 focus:bg-surface-100 dark:focus:bg-surface-800 focus:outline-none border-b border-surface-100 dark:border-surface-800 last:border-b-0"
          @click="onPick(c)"
        >
          <div class="flex items-center gap-2 text-sm font-medium">
            <span class="font-mono text-xs opacity-70">{{ '[' + c.key + ']' }}</span>
            <span>{{ store.labelFor(c) }}</span>
          </div>
          <div class="text-xs opacity-70 line-clamp-1">
            {{ c.fields.title ?? '' }}
          </div>
        </button>
        <div v-if="filtered.length === 0" class="p-4 text-center text-sm opacity-60">
          {{ t('bibliography.pickerNoMatches') }}
        </div>
      </div>
    </div>

    <template #footer>
      <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
    </template>
  </Dialog>
</template>
