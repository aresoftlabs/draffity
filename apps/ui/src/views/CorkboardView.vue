<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Tag from 'primevue/tag';
import type { DocNode } from '@draffity/shared-types';
import { countWords } from '@/stores/document';

const props = defineProps<{
  documents: DocNode[];
  selectedId: string | null;
}>();

const emit = defineEmits<{
  select: [id: string];
}>();

const { t } = useI18n();

// Render documents in binder order (parent_id, then position) — same source
// of truth as Binder so card order matches the tree the user expects.
const ordered = computed(() => {
  return [...props.documents].sort((a, b) => {
    const pa = a.parentId ?? '';
    const pb = b.parentId ?? '';
    if (pa !== pb) return pa.localeCompare(pb);
    return a.position - b.position;
  });
});

function statusDotClass(s: string): string {
  switch (s) {
    case 'revised':
      return 'bg-blue-400';
    case 'final':
      return 'bg-emerald-500';
    case 'trashed':
      return 'bg-rose-400 opacity-60';
    default:
      return 'bg-surface-300 dark:bg-surface-600';
  }
}

function wordCountOf(d: DocNode): number {
  return countWords(d.content ?? '');
}

function progressOf(d: DocNode): number | null {
  const g = d.goalWords ?? null;
  if (!g || g <= 0) return null;
  return Math.min(100, Math.round((wordCountOf(d) / g) * 100));
}

function isFolder(d: DocNode): boolean {
  return d.docType === 'folder';
}
</script>

<template>
  <div class="h-full overflow-auto p-6 bg-surface-50/40 dark:bg-surface-900/40">
    <div v-if="ordered.length === 0" class="h-full flex items-center justify-center opacity-60">
      {{ t('project.noDocuments') }}
    </div>
    <ul
      v-else
      class="grid gap-4"
      :style="{ gridTemplateColumns: 'repeat(auto-fill, minmax(16rem, 1fr))' }"
    >
      <li v-for="doc in ordered" :key="doc.id">
        <button
          type="button"
          class="w-full text-left p-4 rounded-lg border bg-surface-0 dark:bg-surface-950 transition-shadow hover:shadow-md min-h-[10rem] flex flex-col gap-2"
          :class="
            doc.id === selectedId
              ? 'border-primary-500 ring-2 ring-primary-200 dark:ring-primary-900'
              : 'border-surface-200 dark:border-surface-700'
          "
          @click="emit('select', doc.id)"
        >
          <header class="flex items-start gap-2">
            <span
              class="w-1.5 h-1.5 rounded-full mt-2 shrink-0"
              :class="statusDotClass(doc.status)"
              :aria-label="t(`status.${doc.status}`)"
            />
            <h3 class="flex-1 font-serif font-semibold text-sm leading-snug truncate">
              {{ doc.title || t('project.untitled') }}
            </h3>
            <i v-if="isFolder(doc)" class="pi pi-folder text-xs opacity-60" />
          </header>

          <p v-if="doc.synopsis" class="text-xs leading-relaxed opacity-80 line-clamp-4 flex-1">
            {{ doc.synopsis }}
          </p>
          <p v-else class="text-xs italic opacity-40 flex-1">
            {{ t('corkboard.noSynopsis') }}
          </p>

          <footer class="flex items-center justify-between gap-2 mt-auto">
            <span class="text-xs font-mono opacity-70">
              {{ wordCountOf(doc) }}<span v-if="doc.goalWords">/{{ doc.goalWords }}</span>
            </span>
            <div v-if="progressOf(doc) !== null" class="flex-1 mx-2">
              <div class="h-1 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
                <div
                  class="h-full bg-primary-400 transition-[width]"
                  :style="{ width: (progressOf(doc) ?? 0) + '%' }"
                  role="progressbar"
                  :aria-valuenow="progressOf(doc) ?? 0"
                  aria-valuemin="0"
                  aria-valuemax="100"
                />
              </div>
            </div>
            <div v-if="doc.tags.length > 0" class="flex gap-1 flex-wrap justify-end">
              <Tag
                v-for="tag in doc.tags.slice(0, 3)"
                :key="tag"
                :value="tag"
                severity="secondary"
                class="!text-[10px] !py-0 !px-1.5"
              />
              <span v-if="doc.tags.length > 3" class="text-xs opacity-60">
                +{{ doc.tags.length - 3 }}
              </span>
            </div>
          </footer>
        </button>
      </li>
    </ul>
  </div>
</template>
