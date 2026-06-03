<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import type { CodexEntry } from '@draffity/shared-types';
import { useCodexStore } from '@/stores/codex';

const props = defineProps<{
  visible: boolean;
  projectId: string | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  pick: [payload: { id: string; name: string }];
}>();

const { t } = useI18n();
const store = useCodexStore();

const query = ref('');

watch(
  () => [props.visible, props.projectId] as const,
  async ([v, pid]) => {
    if (!v || !pid) return;
    query.value = '';
    await store.loadFor(pid);
  },
);

const filtered = computed<CodexEntry[]>(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return store.entries;
  return store.entries.filter((e) => {
    if (e.name.toLowerCase().includes(q)) return true;
    if ((e.body ?? '').toLowerCase().includes(q)) return true;
    return e.tags.some((t) => t.toLowerCase().includes(q));
  });
});

function close() {
  emit('update:visible', false);
}

function onPick(e: CodexEntry) {
  emit('pick', { id: e.id, name: e.name });
  close();
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('codex.refPickerTitle')"
    :style="{ width: '32rem', maxHeight: '80vh' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-3">
      <p class="text-sm opacity-70">{{ t('codex.refPickerSubtitle') }}</p>
      <InputText
        v-model="query"
        :placeholder="t('codex.refPickerSearch')"
        class="w-full"
        autofocus
      />
      <div
        class="flex flex-col gap-1 max-h-[50vh] overflow-y-auto rounded border border-surface-200 dark:border-surface-700"
      >
        <button
          v-for="e in filtered"
          :key="e.id"
          type="button"
          class="text-left p-2 hover:bg-surface-100 dark:hover:bg-surface-800 focus:bg-surface-100 dark:focus:bg-surface-800 focus:outline-none border-b border-surface-100 dark:border-surface-800 last:border-b-0"
          @click="onPick(e)"
        >
          <div class="flex items-center gap-2 text-sm font-medium">
            <span>{{ e.name }}</span>
            <span class="text-[10px] uppercase tracking-wide opacity-60 ml-auto">
              {{ t(`codex.kind.${e.kind}`) }}
            </span>
          </div>
          <div v-if="e.body" class="text-xs opacity-70 line-clamp-1">{{ e.body }}</div>
        </button>
        <div v-if="filtered.length === 0" class="p-4 text-center text-sm opacity-60">
          {{ t('codex.pickerNoMatches') }}
        </div>
      </div>
    </div>
    <template #footer>
      <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
    </template>
  </Dialog>
</template>
