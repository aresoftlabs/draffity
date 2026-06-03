import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { Citation } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

/** Per-project bibliography cache. Pinia is the right home because the
 *  picker + the citation node both consume it; both want the same in-memory
 *  list rather than each fetching independently. */
export const useCitationsStore = defineStore('citations', () => {
  const projectId = ref<string | null>(null);
  const list = ref<Citation[]>([]);

  const byKey = computed(() => {
    const m = new Map<string, Citation>();
    for (const c of list.value) m.set(c.key, c);
    return m;
  });

  async function loadFor(pid: string) {
    if (projectId.value === pid && list.value.length > 0) return;
    projectId.value = pid;
    list.value = await ipc.listCitations(pid);
  }

  async function refresh() {
    if (!projectId.value) return;
    list.value = await ipc.listCitations(projectId.value);
  }

  function setList(next: Citation[]) {
    list.value = next;
  }

  function reset() {
    projectId.value = null;
    list.value = [];
  }

  /** Pre-render the inline label `(Surname, Year)` for the editor node. */
  function labelFor(c: Citation): string {
    const surname = surnameOf(c);
    const year = c.fields.year ?? '';
    if (surname && year) return `(${surname}, ${year})`;
    if (surname) return `(${surname})`;
    if (year) return `(${year})`;
    return `[${c.key}]`;
  }

  return {
    projectId,
    list,
    byKey,
    loadFor,
    refresh,
    setList,
    reset,
    labelFor,
  };
});

function surnameOf(c: Citation): string {
  const author = c.fields.author ?? '';
  if (!author) return '';
  // First author only â€” BibTeX joins with " and ".
  const first = author.split(' and ')[0].trim();
  if (first.includes(',')) {
    return first.split(',')[0].trim();
  }
  const tokens = first.split(/\s+/);
  return tokens[tokens.length - 1] ?? first;
}
