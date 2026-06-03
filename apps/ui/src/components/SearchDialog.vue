<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import type { SearchHit } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';
import { sanitizeSearchExcerpt } from '@/services/sanitizeHtml';

const props = defineProps<{
  visible: boolean;
  projectId: string;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  jump: [documentId: string];
}>();

const { t } = useI18n();
const { run } = useIpcError();

const query = ref('');
const results = ref<SearchHit[]>([]);
const loading = ref(false);
const debounceMs = 180;
let timer: ReturnType<typeof setTimeout> | null = null;
const inputRef = ref<InstanceType<typeof InputText> | null>(null);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      query.value = '';
      results.value = [];
      void nextTick(() => focusInput());
    } else if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  },
);

watch(query, () => {
  if (timer) clearTimeout(timer);
  if (!query.value.trim()) {
    results.value = [];
    loading.value = false;
    return;
  }
  loading.value = true;
  timer = setTimeout(runSearch, debounceMs);
});

async function runSearch() {
  const q = query.value.trim();
  if (!q) {
    results.value = [];
    loading.value = false;
    return;
  }
  const hits = await run(t('search.error'), () =>
    ipc.searchDocuments({ projectId: props.projectId, query: q }),
  );
  results.value = hits ?? [];
  loading.value = false;
}

function focusInput() {
  // PrimeVue InputText exposes the underlying <input> as $el.
  const inst = inputRef.value as unknown as { $el?: HTMLInputElement };
  inst?.$el?.focus();
}

function close() {
  emit('update:visible', false);
}

function pick(hit: SearchHit) {
  emit('jump', hit.documentId);
  close();
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') close();
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :show-header="false"
    :style="{ width: '40rem' }"
    :pt="{ content: { class: '!p-0' } }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
    @keydown="onKey"
  >
    <div
      class="flex items-center gap-2 px-4 py-3 border-b border-surface-200 dark:border-surface-700"
    >
      <i class="pi pi-search opacity-60" />
      <InputText
        ref="inputRef"
        v-model="query"
        :placeholder="t('search.placeholder')"
        class="flex-1 !border-0 !bg-transparent !shadow-none focus:!ring-0"
      />
      <span class="text-xs opacity-50">Esc</span>
    </div>
    <div class="max-h-[26rem] overflow-auto">
      <p v-if="loading && results.length === 0" class="px-4 py-3 text-sm opacity-60">
        {{ t('search.loading') }}
      </p>
      <p
        v-else-if="!loading && query.trim() && results.length === 0"
        class="px-4 py-3 text-sm opacity-60"
      >
        {{ t('search.empty') }}
      </p>
      <p v-else-if="!query.trim()" class="px-4 py-3 text-sm opacity-60">
        {{ t('search.hint') }}
      </p>
      <ul v-else class="divide-y divide-surface-200 dark:divide-surface-700">
        <li v-for="hit in results" :key="hit.documentId">
          <button
            type="button"
            class="w-full text-left px-4 py-3 hover:bg-surface-100 dark:hover:bg-surface-800 flex flex-col gap-1"
            @click="pick(hit)"
          >
            <span class="text-sm font-medium truncate">{{ hit.title }}</span>
            <!-- FTS5 snippet indexes raw document HTML; sanitize so only the
                 <mark> highlight renders and any imported markup stays inert. -->
            <!-- eslint-disable-next-line vue/no-v-html -->
            <span
              class="text-xs opacity-70 line-clamp-2"
              v-html="sanitizeSearchExcerpt(hit.excerpt)"
            />
          </button>
        </li>
      </ul>
    </div>
  </Dialog>
</template>

<style scoped>
:deep(mark) {
  background: rgb(var(--p-yellow-200) / 0.6);
  color: inherit;
  padding: 0 2px;
  border-radius: 2px;
}
</style>
