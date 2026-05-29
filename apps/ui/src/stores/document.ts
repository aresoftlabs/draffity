import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { DocNode, DocumentInput, DocumentStatus } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

export type SaveState = 'idle' | 'saving' | 'saved' | 'error';

export const useDocumentStore = defineStore('document', () => {
  const documents = ref<DocNode[]>([]);
  const selectedId = ref<string | null>(null);
  const saveState = ref<SaveState>('idle');
  const lastSavedAt = ref<number | null>(null);

  const selected = computed(() =>
    selectedId.value ? (documents.value.find((d) => d.id === selectedId.value) ?? null) : null,
  );

  /** Build the parent → children map for tree views. */
  const tree = computed(() => {
    const byParent = new Map<string | null, DocNode[]>();
    for (const d of documents.value) {
      const key = d.parentId ?? null;
      const arr = byParent.get(key) ?? [];
      arr.push(d);
      byParent.set(key, arr);
    }
    for (const arr of byParent.values()) arr.sort((a, b) => a.position - b.position);
    return byParent;
  });

  const wordCount = computed(() => {
    if (!selected.value) return 0;
    return countWords(selected.value.content ?? '');
  });

  const totalWordCount = computed(() => {
    let n = 0;
    for (const d of documents.value) n += countWords(d.content ?? '');
    return n;
  });

  async function loadFor(projectId: string) {
    documents.value = await ipc.listDocuments(projectId);
    if (!documents.value.find((d) => d.id === selectedId.value)) {
      selectedId.value = documents.value[0]?.id ?? null;
    }
  }

  function select(id: string | null) {
    selectedId.value = id;
  }

  async function create(input: DocumentInput): Promise<DocNode> {
    const doc = await ipc.createDocument(input);
    documents.value = [...documents.value, doc];
    selectedId.value = doc.id;
    return doc;
  }

  async function save(
    id: string,
    patch: { title?: string; content?: string; contentJson?: string },
  ) {
    saveState.value = 'saving';
    try {
      const updated = await ipc.updateDocument({ id, ...patch });
      const idx = documents.value.findIndex((d) => d.id === id);
      if (idx !== -1) documents.value[idx] = updated;
      saveState.value = 'saved';
      lastSavedAt.value = Date.now();
    } catch (e) {
      saveState.value = 'error';
      throw e;
    }
  }

  async function remove(id: string) {
    await ipc.deleteDocument(id);
    documents.value = documents.value.filter((d) => d.id !== id);
    if (selectedId.value === id) selectedId.value = documents.value[0]?.id ?? null;
  }

  async function move(id: string, parentId: string | null, position: number) {
    await ipc.moveDocument({ id, parentId, position });
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) {
      documents.value[idx] = {
        ...documents.value[idx],
        parentId: parentId ?? null,
        position,
      };
    }
  }

  async function setStatus(id: string, status: DocumentStatus) {
    const updated = await ipc.setDocumentStatus({ id, status });
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  async function setTags(id: string, tags: string[]) {
    const updated = await ipc.setDocumentTags({ id, tags });
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  async function setLabels(id: string, labelIds: string[]) {
    const updated = await ipc.setDocumentLabels(id, labelIds);
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  async function setMetadata(id: string, fieldId: string, value: string | null) {
    const updated = await ipc.setDocumentMetadata(id, fieldId, value);
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  async function setGoal(id: string, goal: number | null) {
    const updated = await ipc.setDocumentGoal({ id, goal });
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  async function setSynopsis(id: string, synopsis: string | null) {
    const updated = await ipc.setDocumentSynopsis({ id, synopsis });
    const idx = documents.value.findIndex((d) => d.id === id);
    if (idx !== -1) documents.value[idx] = updated;
  }

  /** Persist a binder reorder. Apply ops sequentially (1-2 in practice:
   * the new parent and, if the node changed parents, also the old one).
   * Reloads documents after to converge with the server's view. */
  async function reorder(projectId: string, ops: ReorderOp[]) {
    for (const op of ops) {
      await ipc.reorderDocuments({
        projectId,
        parentId: op.parentId,
        orderedIds: op.orderedIds,
      });
    }
    await loadFor(projectId);
  }

  function reset() {
    documents.value = [];
    selectedId.value = null;
    saveState.value = 'idle';
    lastSavedAt.value = null;
  }

  return {
    documents,
    selectedId,
    saveState,
    lastSavedAt,
    selected,
    tree,
    wordCount,
    totalWordCount,
    loadFor,
    select,
    create,
    save,
    remove,
    move,
    reorder,
    setStatus,
    setTags,
    setLabels,
    setMetadata,
    setGoal,
    setSynopsis,
    reset,
  };
});

export type ReorderOp = {
  parentId: string | null;
  orderedIds: string[];
};

/** Plain-text word counter that strips HTML tags. */
export function countWords(htmlOrText: string): number {
  if (!htmlOrText) return 0;
  const text = htmlOrText.replace(/<[^>]*>/g, ' ').replace(/&nbsp;/g, ' ');
  const matches = text.match(/\S+/g);
  return matches ? matches.length : 0;
}
