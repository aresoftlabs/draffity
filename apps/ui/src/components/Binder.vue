<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Tree from 'primevue/tree';
import type { TreeNode } from 'primevue/treenode';
import Button from 'primevue/button';
import Menu from 'primevue/menu';
import { ref } from 'vue';
import type { DocNode, DocumentType } from '@draffity/shared-types';

const props = defineProps<{
  documents: DocNode[];
  selectedId: string | null;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  create: [type: DocumentType];
  remove: [id: string];
  move: [payload: { id: string; parentId: string | null; position: number }];
}>();

const { t } = useI18n();

function buildNodes(parentId: string | null): TreeNode[] {
  return props.documents
    .filter((d) => (d.parentId ?? null) === parentId)
    .sort((a, b) => a.position - b.position)
    .map((d) => {
      const children = buildNodes(d.id);
      const node: TreeNode = {
        key: d.id,
        label: d.title || t('project.untitled'),
        data: d,
        icon: iconFor(d.docType),
        children: children.length ? children : undefined,
      };
      return node;
    });
}

function iconFor(t: DocumentType): string {
  switch (t) {
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
  }
}

const nodes = computed(() => buildNodes(null));

const selectionKeys = computed<Record<string, boolean>>(() => {
  return props.selectedId ? { [props.selectedId]: true } : {};
});

function onNodeSelect(node: TreeNode) {
  if (typeof node.key === 'string') emit('select', node.key);
}

const newMenu = ref();
const newMenuItems = computed(() => [
  {
    label: t('project.newChapter'),
    icon: 'pi pi-book',
    command: () => emit('create', 'chapter'),
  },
  {
    label: t('project.newScene'),
    icon: 'pi pi-bookmark',
    command: () => emit('create', 'scene'),
  },
  {
    label: t('project.newNote'),
    icon: 'pi pi-sticky-note',
    command: () => emit('create', 'note'),
  },
]);
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-900">
    <header
      class="flex items-center justify-between px-3 py-2 border-b border-surface-200 dark:border-surface-700"
    >
      <h3 class="text-sm font-semibold uppercase tracking-wide opacity-70">
        {{ t('project.binder') }}
      </h3>
      <div class="flex items-center gap-1">
        <Button
          v-if="!readOnly"
          icon="pi pi-plus"
          text
          severity="secondary"
          size="small"
          :aria-label="t('project.newDocument')"
          @click="(e) => newMenu?.toggle(e)"
        />
        <Menu ref="newMenu" :model="newMenuItems" popup />
      </div>
    </header>

    <div v-if="documents.length === 0" class="p-4 text-sm opacity-60">
      {{ t('project.noDocuments') }}
    </div>

    <Tree
      v-else
      :value="nodes"
      :selection-keys="selectionKeys"
      selection-mode="single"
      class="flex-1 overflow-auto !border-0 !bg-transparent !p-1"
      @node-select="onNodeSelect"
    >
      <template #default="{ node }">
        <span class="text-sm truncate">{{ node.label }}</span>
      </template>
    </Tree>
  </div>
</template>

<style scoped>
:deep(.p-tree) {
  background: transparent;
  border: 0;
}
:deep(.p-tree .p-treenode-content) {
  padding: 4px 6px;
  border-radius: 4px;
}
</style>
