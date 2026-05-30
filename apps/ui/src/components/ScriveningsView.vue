<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { DocNode } from '@draffity/shared-types';

const props = defineProps<{
  folder: DocNode;
  documents: DocNode[];
}>();

const { t } = useI18n();

/** All descendants of `folder` in DFS pre-order, respecting position. */
const descendants = computed<DocNode[]>(() => {
  const byParent = new Map<string, DocNode[]>();
  for (const d of props.documents) {
    const key = d.parentId ?? '';
    const list = byParent.get(key) ?? [];
    list.push(d);
    byParent.set(key, list);
  }
  for (const list of byParent.values()) {
    list.sort((a, b) => a.position - b.position);
  }

  const out: DocNode[] = [];
  function walk(parentId: string) {
    const children = byParent.get(parentId) ?? [];
    for (const child of children) {
      out.push(child);
      walk(child.id);
    }
  }
  walk(props.folder.id);
  return out;
});

const leaves = computed(() => descendants.value.filter((d) => d.docType !== 'folder'));
</script>

<template>
  <div class="h-full overflow-auto bg-surface-0 dark:bg-surface-950">
    <article class="scrivenings-host px-10 py-8 max-w-3xl mx-auto">
      <header class="mb-6 pb-4 border-b border-surface-200 dark:border-surface-700">
        <p class="text-xs uppercase tracking-wide opacity-60">{{ t('scrivenings.label') }}</p>
        <h1 class="text-2xl font-display font-bold mt-1">
          {{ folder.title || t('project.untitled') }}
        </h1>
        <p v-if="folder.synopsis" class="text-sm italic opacity-70 mt-2">
          {{ folder.synopsis }}
        </p>
        <p class="text-xs opacity-60 mt-2">
          {{ t('scrivenings.summary', { count: leaves.length }) }}
        </p>
      </header>

      <section v-for="(doc, i) in descendants" :key="doc.id" class="scrivenings-item">
        <h2
          v-if="doc.docType === 'folder'"
          class="text-lg font-display font-bold mt-8 mb-2 text-primary-600 dark:text-primary-300"
        >
          {{ doc.title || t('project.untitled') }}
        </h2>
        <template v-else>
          <hr
            v-if="i > 0"
            class="my-8 border-0 border-t border-surface-200 dark:border-surface-700"
          />
          <h3 class="text-sm font-semibold uppercase tracking-wide opacity-60 mb-3">
            {{ doc.title || t('project.untitled') }}
          </h3>
          <!-- TipTap HTML is sourced from our own storage; safe to render. -->
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div
            v-if="doc.content && doc.content.trim().length > 0"
            class="prose-style scrivenings-content"
            v-html="doc.content"
          />
          <p v-else class="text-sm italic opacity-40">{{ t('scrivenings.empty') }}</p>
        </template>
      </section>

      <p v-if="descendants.length === 0" class="text-sm italic opacity-60 text-center py-12">
        {{ t('scrivenings.noChildren') }}
      </p>
    </article>
  </div>
</template>

<style scoped>
.scrivenings-host :deep(.scrivenings-content) {
  font-family: Lora, Georgia, 'Times New Roman', serif;
  font-size: 17px;
  line-height: 1.7;
}
.scrivenings-host :deep(.scrivenings-content h1) {
  font-size: 1.75em;
  font-weight: 700;
  margin: 1em 0 0.5em;
}
.scrivenings-host :deep(.scrivenings-content h2) {
  font-size: 1.35em;
  font-weight: 700;
  margin: 1em 0 0.5em;
}
.scrivenings-host :deep(.scrivenings-content p) {
  margin: 0 0 1em;
}
.scrivenings-host :deep(.scrivenings-content blockquote) {
  border-left: 3px solid var(--p-surface-300, #cbd5e1);
  padding-left: 1em;
  color: var(--p-surface-700, #475569);
  margin: 1em 0;
}
.scrivenings-host :deep(.scrivenings-content ul) {
  list-style: disc;
  padding-left: 1.4em;
  margin: 0 0 1em;
}
.scrivenings-host :deep(.scrivenings-content ol) {
  list-style: decimal;
  padding-left: 1.4em;
  margin: 0 0 1em;
}
</style>
