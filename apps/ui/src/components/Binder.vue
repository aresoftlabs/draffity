<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Tree, { type TreeNodeDropEvent } from 'primevue/tree';
import type { TreeNode } from 'primevue/treenode';
import Button from 'primevue/button';
import Menu from 'primevue/menu';
import Select from 'primevue/select';
import { ref } from 'vue';
import type { DocNode, DocumentStatus, DocumentType } from '@draffity/shared-types';
import type { ReorderOp } from '@/stores/document';

const props = defineProps<{
  documents: DocNode[];
  selectedId: string | null;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  create: [type: DocumentType];
  remove: [id: string];
  reorder: [ops: ReorderOp[]];
}>();

const tagFilter = ref<string | null>(null);

const availableTags = computed(() => {
  const set = new Set<string>();
  for (const d of props.documents) {
    for (const t of d.tags) set.add(t);
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b));
});

/** Docs that pass the current tag filter. When a tag is selected we also
 * keep its ancestors so the tree can render the path (folders without the
 * tag stay visible as scaffolding). */
const visibleIds = computed(() => {
  if (!tagFilter.value) return null;
  const direct = new Set<string>();
  for (const d of props.documents) {
    if (d.tags.includes(tagFilter.value!)) direct.add(d.id);
  }
  // Walk ancestors of each matching doc and add them.
  const byId = new Map(props.documents.map((d) => [d.id, d] as const));
  const out = new Set<string>(direct);
  for (const id of direct) {
    let cursor = byId.get(id)?.parentId ?? null;
    while (cursor) {
      if (out.has(cursor)) break;
      out.add(cursor);
      cursor = byId.get(cursor)?.parentId ?? null;
    }
  }
  return out;
});

const { t } = useI18n();

function buildNodes(parentId: string | null): TreeNode[] {
  const allowed = visibleIds.value;
  return props.documents
    .filter((d) => (d.parentId ?? null) === parentId)
    .filter((d) => allowed === null || allowed.has(d.id))
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

/** Class for the small status dot rendered next to the node label. */
function statusDotClass(s: DocumentStatus): string {
  switch (s) {
    case 'revised':
      return 'bg-blue-400';
    case 'final':
      return 'bg-emerald-500';
    case 'trashed':
      return 'bg-rose-400 opacity-60';
    case 'draft':
    default:
      return 'bg-surface-300 dark:bg-surface-600';
  }
}

const nodes = computed(() => buildNodes(null));

const selectionKeys = computed<Record<string, boolean>>(() => {
  return props.selectedId ? { [props.selectedId]: true } : {};
});

function onNodeSelect(node: TreeNode) {
  if (typeof node.key === 'string') emit('select', node.key);
}

/** Locate `targetKey` inside `tree`, returning its parent node (or null if
 * at root) plus the sibling list it now belongs to. */
function findContext(
  tree: TreeNode[],
  targetKey: string,
  parent: TreeNode | null = null,
): { parent: TreeNode | null; siblings: TreeNode[] } | null {
  for (const node of tree) {
    if (node.key === targetKey) return { parent, siblings: tree };
    if (node.children) {
      const found = findContext(node.children, targetKey, node);
      if (found) return found;
    }
  }
  return null;
}

/** Find the children list of a node identified by key. */
function findChildren(tree: TreeNode[], parentKey: string): TreeNode[] {
  for (const node of tree) {
    if (node.key === parentKey) return node.children ?? [];
    if (node.children) {
      const inner = findChildren(node.children, parentKey);
      if (inner.length > 0 || node.children.some((c) => c.key === parentKey)) {
        return inner;
      }
    }
  }
  return [];
}

function onNodeDrop(event: TreeNodeDropEvent) {
  if (props.readOnly) return;
  const dragId = typeof event.dragNode.key === 'string' ? event.dragNode.key : null;
  if (!dragId) return;

  const oldDoc = props.documents.find((d) => d.id === dragId);
  if (!oldDoc) return;
  const oldParentId = oldDoc.parentId ?? null;

  const ctx = findContext(event.value, dragId);
  if (!ctx) return;
  const newParentId = ctx.parent && typeof ctx.parent.key === 'string' ? ctx.parent.key : null;
  const newOrderedIds = ctx.siblings
    .map((s) => (typeof s.key === 'string' ? s.key : null))
    .filter((k): k is string => k !== null);

  const ops: ReorderOp[] = [{ parentId: newParentId, orderedIds: newOrderedIds }];

  // When the node changed parents, the old parent's remaining children
  // also need their positions compacted.
  if (oldParentId !== newParentId) {
    const oldSiblings = oldParentId === null ? event.value : findChildren(event.value, oldParentId);
    const oldOrderedIds = oldSiblings
      .map((s) => (typeof s.key === 'string' ? s.key : null))
      .filter((k): k is string => k !== null);
    ops.push({ parentId: oldParentId, orderedIds: oldOrderedIds });
  }

  emit('reorder', ops);
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

    <div
      v-if="availableTags.length > 0"
      class="px-3 py-2 border-b border-surface-200 dark:border-surface-700"
    >
      <Select
        v-model="tagFilter"
        :options="availableTags"
        :placeholder="t('tags.filterPlaceholder')"
        :show-clear="true"
        class="w-full !text-xs"
        size="small"
      />
    </div>

    <div v-if="documents.length === 0" class="p-4 text-sm opacity-60">
      {{ t('project.noDocuments') }}
    </div>

    <Tree
      v-else
      :value="nodes"
      :selection-keys="selectionKeys"
      selection-mode="single"
      :dragdrop-scope="readOnly ? undefined : 'documents'"
      class="flex-1 overflow-auto !border-0 !bg-transparent !p-1"
      @node-select="onNodeSelect"
      @node-drop="onNodeDrop"
    >
      <template #default="{ node }">
        <span class="flex items-center gap-2 min-w-0">
          <span
            v-if="node.data"
            class="w-1.5 h-1.5 rounded-full shrink-0"
            :class="statusDotClass((node.data as DocNode).status)"
            :aria-label="t(`status.${(node.data as DocNode).status}`)"
          />
          <span class="text-sm truncate">{{ node.label }}</span>
        </span>
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
