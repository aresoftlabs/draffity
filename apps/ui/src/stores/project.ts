import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { Project, ProjectInput } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([]);
  const activeId = ref<string | null>(null);
  const loading = ref(false);

  const active = computed(() => projects.value.find((p) => p.status === 'active') ?? null);
  const archived = computed(() => projects.value.filter((p) => p.status === 'archived'));
  const current = computed(() =>
    activeId.value ? (projects.value.find((p) => p.id === activeId.value) ?? null) : null,
  );
  const isCurrentReadOnly = computed(() => current.value?.status === 'archived');

  async function loadAll() {
    loading.value = true;
    try {
      projects.value = await ipc.listProjects();
      const a = await ipc.getActiveProject();
      activeId.value = a?.id ?? null;
    } finally {
      loading.value = false;
    }
  }

  async function create(input: ProjectInput): Promise<Project> {
    const project = await ipc.createProject(input);
    await loadAll();
    activeId.value = project.id;
    return project;
  }

  async function open(id: string): Promise<Project> {
    const project = await ipc.openProject(id);
    await loadAll();
    activeId.value = project.id;
    return project;
  }

  async function archive(id: string): Promise<void> {
    await ipc.archiveProject(id);
    await loadAll();
    if (activeId.value === id) activeId.value = null;
  }

  async function remove(id: string): Promise<void> {
    await ipc.deleteProject(id);
    await loadAll();
    if (activeId.value === id) activeId.value = null;
  }

  async function setGoal(id: string, goal: number | null) {
    const updated = await ipc.setProjectGoal({ id, goal });
    const idx = projects.value.findIndex((p) => p.id === id);
    if (idx !== -1) projects.value[idx] = updated;
  }

  function selectLocally(id: string | null) {
    activeId.value = id;
  }

  return {
    projects,
    activeId,
    loading,
    active,
    archived,
    current,
    isCurrentReadOnly,
    loadAll,
    create,
    open,
    archive,
    remove,
    setGoal,
    selectLocally,
  };
});
