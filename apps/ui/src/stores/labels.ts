import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { Label, LabelInput } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

/** Client-side cache of a project's label definitions (I-05/I-06). The
 *  backend (SQLite) is the source of truth; this store mirrors it so the
 *  binder / outliner / corkboard / inspector can resolve `DocNode.labelIds`
 *  to a name + color without each component fetching independently. */
export const useLabelStore = defineStore('labels', () => {
  const labels = ref<Label[]>([]);
  const projectId = ref<string | null>(null);

  /** id → Label lookup for chip rendering. */
  const byId = computed(() => new Map(labels.value.map((l) => [l.id, l] as const)));

  async function loadFor(pid: string) {
    projectId.value = pid;
    labels.value = await ipc.listLabels(pid);
  }

  async function create(input: LabelInput): Promise<Label> {
    const label = await ipc.createLabel(input);
    labels.value = [...labels.value, label].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
    );
    return label;
  }

  async function update(id: string, name: string, color: string): Promise<Label> {
    const updated = await ipc.updateLabel(id, name, color);
    const idx = labels.value.findIndex((l) => l.id === id);
    if (idx !== -1) labels.value[idx] = updated;
    labels.value = [...labels.value].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
    );
    return updated;
  }

  async function remove(id: string) {
    await ipc.deleteLabel(id);
    labels.value = labels.value.filter((l) => l.id !== id);
  }

  function reset() {
    labels.value = [];
    projectId.value = null;
  }

  return { labels, byId, loadFor, create, update, remove, reset };
});
