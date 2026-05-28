import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { CodexEntry, CodexInput, CodexKind, CodexUpdate } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

/** Per-project codex cache. Re-fetches whenever `loadFor` is called with a
 *  different project id — same shape as the citations store. */
export const useCodexStore = defineStore('codex', () => {
  const projectId = ref<string | null>(null);
  const entries = ref<CodexEntry[]>([]);

  const byId = computed(() => {
    const m = new Map<string, CodexEntry>();
    for (const e of entries.value) m.set(e.id, e);
    return m;
  });

  /** Lowercase-name lookup. The [[cross-ref]] picker uses this to resolve
   *  a name typed in the editor to a real entry id. Conflicts (two entries
   *  sharing a name) resolve to the first by alphabetical order. */
  const byNameLower = computed(() => {
    const m = new Map<string, CodexEntry>();
    for (const e of entries.value) {
      const k = e.name.toLowerCase();
      if (!m.has(k)) m.set(k, e);
    }
    return m;
  });

  /** Distinct tags across the catalogue, sorted alphabetical. Used by the
   *  filter dropdown in the codex panel. */
  const allTags = computed(() => {
    const s = new Set<string>();
    for (const e of entries.value) for (const t of e.tags) s.add(t);
    return [...s].sort((a, b) => a.localeCompare(b));
  });

  async function loadFor(pid: string) {
    if (projectId.value === pid && entries.value.length > 0) return;
    projectId.value = pid;
    entries.value = await ipc.listCodexEntries(pid);
  }

  async function refresh() {
    if (!projectId.value) return;
    entries.value = await ipc.listCodexEntries(projectId.value);
  }

  async function create(input: CodexInput): Promise<CodexEntry> {
    const e = await ipc.createCodexEntry(input);
    // Insert at the right place alphabetically so the panel doesn't jump.
    const next = [...entries.value, e].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
    );
    entries.value = next;
    return e;
  }

  async function update(id: string, patch: CodexUpdate): Promise<CodexEntry> {
    const e = await ipc.updateCodexEntry({ id, patch });
    const idx = entries.value.findIndex((x) => x.id === id);
    if (idx === -1) {
      entries.value = [...entries.value, e].sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
      );
    } else {
      // Replace + re-sort in case the rename moved its alphabetical slot.
      const copy = [...entries.value];
      copy[idx] = e;
      copy.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
      entries.value = copy;
    }
    return e;
  }

  async function remove(id: string) {
    await ipc.deleteCodexEntry(id);
    entries.value = entries.value.filter((e) => e.id !== id);
  }

  function reset() {
    projectId.value = null;
    entries.value = [];
  }

  function filtered(opts: { query: string; kind: CodexKind | null; tag: string | null }) {
    const q = opts.query.trim().toLowerCase();
    return entries.value.filter((e) => {
      if (opts.kind && e.kind !== opts.kind) return false;
      if (opts.tag && !e.tags.includes(opts.tag)) return false;
      if (!q) return true;
      if (e.name.toLowerCase().includes(q)) return true;
      if ((e.body ?? '').toLowerCase().includes(q)) return true;
      return e.tags.some((t) => t.toLowerCase().includes(q));
    });
  }

  return {
    projectId,
    entries,
    byId,
    byNameLower,
    allTags,
    loadFor,
    refresh,
    create,
    update,
    remove,
    reset,
    filtered,
  };
});
