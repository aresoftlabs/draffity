<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import InputText from 'primevue/inputtext';
import Select from 'primevue/select';
import Button from 'primevue/button';
import { useConfirm } from 'primevue/useconfirm';
import { useToast } from 'primevue/usetoast';
import type { CodexEntry, CodexKind } from '@draffity/shared-types';
import { useCodexStore } from '@/stores/codex';
import { useIpcError } from '@/composables/useIpcError';
import CodexEntryCard from '@/components/CodexEntryCard.vue';
import CodexEntryDialog from '@/components/CodexEntryDialog.vue';

const props = defineProps<{
  projectId: string;
  readOnly?: boolean;
}>();

const { t } = useI18n();
const { run } = useIpcError();
const store = useCodexStore();
const confirm = useConfirm();
const toast = useToast();

const query = ref('');
const kindFilter = ref<CodexKind | null>(null);
const tagFilter = ref<string | null>(null);

const showDialog = ref(false);
const editing = ref<CodexEntry | null>(null);

const kindOptions = computed(() => [
  { value: null, label: t('codex.filterAllKinds') },
  { value: 'character', label: t('codex.kind.character') },
  { value: 'place', label: t('codex.kind.place') },
  { value: 'object', label: t('codex.kind.object') },
  { value: 'note', label: t('codex.kind.note') },
]);

const tagOptions = computed(() => [
  { value: null, label: t('codex.filterAllTags') },
  ...store.allTags.map((t) => ({ value: t, label: t })),
]);

const visible = computed(() =>
  store.filtered({ query: query.value, kind: kindFilter.value, tag: tagFilter.value }),
);

watch(
  () => props.projectId,
  async (pid) => {
    if (!pid) return;
    await run(t('codex.loadError'), () => store.loadFor(pid));
  },
  { immediate: true },
);

onMounted(() => {
  if (props.projectId) void store.loadFor(props.projectId);
});

function onOpenNew() {
  editing.value = null;
  showDialog.value = true;
}

function onOpenEntry(id: string) {
  const e = store.byId.get(id);
  if (!e) return;
  editing.value = e;
  showDialog.value = true;
}

function onDelete(e: CodexEntry) {
  confirm.require({
    message: t('codex.deleteConfirm', { name: e.name }),
    header: t('codex.deleteTitle'),
    acceptClass: 'p-button-danger',
    accept: async () => {
      await run(t('codex.saveError'), () => store.remove(e.id));
      toast.add({
        severity: 'success',
        summary: t('codex.title'),
        detail: t('codex.deleteSuccess', { name: e.name }),
        life: 3000,
      });
      if (editing.value?.id === e.id) {
        showDialog.value = false;
        editing.value = null;
      }
    },
  });
}
</script>

<template>
  <div class="h-full flex flex-col">
    <div
      class="flex items-center gap-2 p-3 border-b border-surface-200 dark:border-surface-700 bg-surface-50/60 dark:bg-surface-900/60 flex-wrap"
    >
      <InputText
        v-model="query"
        :placeholder="t('codex.searchPlaceholder')"
        class="flex-1 min-w-40"
      />
      <Select
        v-model="kindFilter"
        :options="kindOptions"
        option-label="label"
        option-value="value"
        class="!min-w-40"
      />
      <Select
        v-model="tagFilter"
        :options="tagOptions"
        option-label="label"
        option-value="value"
        class="!min-w-40"
      />
      <Button
        :label="t('codex.newEntry')"
        icon="pi pi-plus"
        :disabled="readOnly"
        @click="onOpenNew"
      />
    </div>

    <div class="flex-1 overflow-auto p-4 bg-surface-50/40 dark:bg-surface-900/40">
      <div
        v-if="visible.length === 0"
        class="h-full flex flex-col items-center justify-center text-sm opacity-60 gap-2"
      >
        <p>{{ store.entries.length === 0 ? t('codex.empty') : t('codex.noMatches') }}</p>
        <Button
          v-if="store.entries.length === 0 && !readOnly"
          :label="t('codex.newEntry')"
          icon="pi pi-plus"
          text
          @click="onOpenNew"
        />
      </div>
      <div v-else class="grid gap-3 grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
        <div v-for="e in visible" :key="e.id" class="relative group">
          <CodexEntryCard :entry="e" @open="onOpenEntry" />
          <button
            v-if="!readOnly"
            type="button"
            class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-surface-100 dark:hover:bg-surface-800"
            :aria-label="t('actions.delete')"
            @click.stop="onDelete(e)"
          >
            <i class="pi pi-trash text-rose-500 text-xs" />
          </button>
        </div>
      </div>
    </div>

    <CodexEntryDialog v-model:visible="showDialog" :project-id="projectId" :entry="editing" />
  </div>
</template>
