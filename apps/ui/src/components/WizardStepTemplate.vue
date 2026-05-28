<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Tag from 'primevue/tag';
import type { Template, TemplateNode } from '@draffity/shared-types';

const props = defineProps<{
  templates: Template[];
  selectedId: string | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  select: [tpl: Template];
}>();

const { t } = useI18n();

const selected = computed(() =>
  props.selectedId ? (props.templates.find((tpl) => tpl.id === props.selectedId) ?? null) : null,
);

const previewNodes = computed(() => (selected.value ? flattenTree(selected.value.structure) : []));

function flattenTree(nodes: TemplateNode[]): { title: string; level: number; type: string }[] {
  const out: { title: string; level: number; type: string }[] = [];
  function walk(items: TemplateNode[], level: number) {
    for (const n of items) {
      out.push({ title: n.title, level, type: n.docType });
      if (n.children?.length) walk(n.children, level + 1);
    }
  }
  walk(nodes, 0);
  return out;
}

function templateBadge(tpl: Template): string {
  return t(`newProject.kind.${tpl.kind}`);
}

function iconForDocType(type: string) {
  switch (type) {
    case 'chapter':
      return 'pi pi-book';
    case 'scene':
      return 'pi pi-bookmark';
    case 'note':
      return 'pi pi-sticky-note';
    case 'folder':
      return 'pi pi-folder';
    case 'manga_page':
      return 'pi pi-image';
    default:
      return 'pi pi-file';
  }
}
</script>

<template>
  <div class="flex flex-col gap-3 min-h-[20rem]">
    <p v-if="loading" class="text-sm opacity-60">…</p>
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <button
        v-for="tpl in templates"
        :key="tpl.id"
        type="button"
        class="text-left p-4 rounded-lg border transition-shadow hover:shadow-sm"
        :class="
          selectedId === tpl.id
            ? 'border-primary-500 ring-2 ring-primary-200 dark:ring-primary-900 bg-primary-50/40 dark:bg-primary-900/10'
            : 'border-surface-200 dark:border-surface-700'
        "
        @click="emit('select', tpl)"
      >
        <div class="flex items-start justify-between gap-2 mb-2">
          <h3 class="font-serif font-semibold text-base">{{ tpl.name }}</h3>
          <Tag :value="templateBadge(tpl)" severity="info" />
        </div>
        <p v-if="tpl.description" class="text-sm opacity-70 leading-relaxed">
          {{ tpl.description }}
        </p>
      </button>
    </div>

    <div v-if="selected" class="mt-4">
      <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('newProject.preview') }}
      </h4>
      <ul
        class="text-sm border border-surface-200 dark:border-surface-700 rounded-md p-3 bg-surface-50 dark:bg-surface-900 max-h-48 overflow-auto space-y-1"
      >
        <li
          v-for="(n, i) in previewNodes"
          :key="i"
          class="flex items-center gap-2"
          :style="{ paddingLeft: `${n.level * 16}px` }"
        >
          <i :class="iconForDocType(n.type)" class="text-xs opacity-60" />
          <span>{{ n.title }}</span>
        </li>
        <li v-if="previewNodes.length === 0" class="opacity-60">
          {{ t('newProject.previewEmpty') }}
        </li>
      </ul>
    </div>
  </div>
</template>
