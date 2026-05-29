import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { CustomField, CustomFieldInput } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

/** Client-side cache of a project's custom metadata field definitions
 *  (I-08/I-09). The backend (SQLite) is the source of truth; this store
 *  mirrors it so the inspector + outliner read one shared list. Ordered by
 *  the backend `position`. */
export const useCustomFieldStore = defineStore('customFields', () => {
  const fields = ref<CustomField[]>([]);
  const projectId = ref<string | null>(null);

  async function loadFor(pid: string) {
    projectId.value = pid;
    fields.value = await ipc.listCustomFields(pid);
  }

  async function create(input: CustomFieldInput): Promise<CustomField> {
    const field = await ipc.createCustomField(input);
    fields.value = [...fields.value, field];
    return field;
  }

  async function update(id: string, name: string, options: string[]): Promise<CustomField> {
    const updated = await ipc.updateCustomField(id, name, options);
    const idx = fields.value.findIndex((f) => f.id === id);
    if (idx !== -1) fields.value[idx] = updated;
    return updated;
  }

  async function remove(id: string) {
    await ipc.deleteCustomField(id);
    fields.value = fields.value.filter((f) => f.id !== id);
  }

  function reset() {
    fields.value = [];
    projectId.value = null;
  }

  return { fields, loadFor, create, update, remove, reset };
});
