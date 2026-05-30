<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import Button from 'primevue/button';
import { useConfirm } from 'primevue/useconfirm';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { ImportFormat, ProjectInput } from '@draffity/shared-types';
import { useProjectStore } from '@/stores/project';
import { useUiStore } from '@/stores/ui';
import { useIpcError } from '@/composables/useIpcError';
import { ipc } from '@/services/ipc';
import ProjectCard from '@/components/ProjectCard.vue';
import NewProjectWizard from '@/components/NewProjectWizard.vue';
import SwitchProjectDialog from '@/components/SwitchProjectDialog.vue';

const { t } = useI18n();
const router = useRouter();
const projectStore = useProjectStore();
const uiStore = useUiStore();
const { active, archived, projects, loading } = storeToRefs(projectStore);
const { run } = useIpcError();
const confirm = useConfirm();

const showNew = ref(false);
const switchTarget = ref<{ id: string; title: string } | null>(null);
const showSwitch = computed({
  get: () => switchTarget.value !== null,
  set: (v) => {
    if (!v) switchTarget.value = null;
  },
});

onMounted(async () => {
  await run(t('errors.loadProjects'), () => projectStore.loadAll());
  // If onboarding asked us to open the wizard right after finishing, do so.
  if (uiStore.consumeNewProjectRequest()) {
    showNew.value = true;
  }
});

async function onCreate(input: ProjectInput) {
  const project = await run(t('errors.createProject'), () => projectStore.create(input));
  if (project) router.push({ name: 'project', params: { id: project.id } });
}

const importing = ref(false);

async function onImport() {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: 'Manuscript', extensions: ['md', 'markdown', 'docx'] },
      { name: 'Markdown', extensions: ['md', 'markdown'] },
      { name: 'Word', extensions: ['docx'] },
    ],
    title: t('dashboard.importProject'),
  });
  if (typeof picked !== 'string') return;
  const ext = picked.toLowerCase().split('.').pop() ?? '';
  const format: ImportFormat | null =
    ext === 'docx' ? 'docx' : ext === 'md' || ext === 'markdown' ? 'markdown' : null;
  if (!format) return;
  importing.value = true;
  try {
    const bytes = await readFile(picked);
    const filenameHint =
      picked
        .split(/[\\/]/)
        .pop()
        ?.replace(/\.[^.]+$/, '') ?? 'imported';
    const project = await run(t('errors.importProject'), () =>
      ipc.importProject({ format, bytes: Array.from(bytes), filenameHint }),
    );
    if (project) {
      await projectStore.loadAll();
      router.push({ name: 'project', params: { id: project.id } });
    }
  } finally {
    importing.value = false;
  }
}

async function onOpenActive(id: string) {
  router.push({ name: 'project', params: { id } });
}

async function onActivateArchived(id: string) {
  const target = projectStore.projects.find((p) => p.id === id);
  if (!target) return;
  if (active.value && active.value.id !== id) {
    switchTarget.value = { id, title: target.title };
  } else {
    await openProject(id);
  }
}

async function openProject(id: string) {
  const project = await run(t('errors.openProject'), () => projectStore.open(id));
  if (project) router.push({ name: 'project', params: { id: project.id } });
}

async function confirmSwitch() {
  if (!switchTarget.value) return;
  const id = switchTarget.value.id;
  switchTarget.value = null;
  await openProject(id);
}

function onDelete(id: string) {
  const project = projects.value.find((p) => p.id === id);
  if (!project) return;
  confirm.require({
    message: `${t('actions.delete')}: ${project.title}?`,
    icon: 'pi pi-exclamation-triangle',
    acceptLabel: t('actions.delete'),
    rejectLabel: t('actions.cancel'),
    acceptProps: { severity: 'danger' },
    accept: async () => {
      await run(t('errors.deleteProject'), () => projectStore.remove(id));
    },
  });
}
</script>

<template>
  <section class="flex-1 min-h-0 overflow-y-auto flex flex-col p-8 max-w-5xl w-full mx-auto gap-8">
    <header class="flex items-end justify-between gap-4 flex-wrap">
      <div>
        <h1 class="text-3xl font-display font-bold">{{ t('dashboard.title') }}</h1>
        <p class="text-sm opacity-70 mt-1">{{ t('dashboard.subtitle') }}</p>
      </div>
      <div class="flex items-center gap-2">
        <Button
          :label="t('dashboard.importProject')"
          icon="pi pi-upload"
          severity="secondary"
          outlined
          :loading="importing"
          @click="onImport"
        />
        <Button :label="t('dashboard.newProject')" icon="pi pi-plus" @click="showNew = true" />
      </div>
    </header>

    <div v-if="loading" class="text-sm opacity-60">…</div>

    <template v-else-if="projects.length === 0">
      <div class="flex-1 flex flex-col items-center justify-center text-center gap-4 opacity-80">
        <i class="pi pi-book text-6xl opacity-30" />
        <p class="text-lg">{{ t('dashboard.empty') }}</p>
        <Button :label="t('dashboard.newProject')" icon="pi pi-plus" @click="showNew = true" />
      </div>
    </template>

    <template v-else>
      <section v-if="active">
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-60 mb-3">
          {{ t('dashboard.active') }}
        </h2>
        <ProjectCard
          :project="active"
          highlighted
          class="md:max-w-md"
          @open="onOpenActive"
          @delete="onDelete"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-60 mb-3">
          {{ t('dashboard.archivedSection') }}
        </h2>
        <p v-if="archived.length === 0" class="text-sm opacity-60">
          {{ t('dashboard.noArchived') }}
        </p>
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <ProjectCard
            v-for="p in archived"
            :key="p.id"
            :project="p"
            @open="onActivateArchived"
            @delete="onDelete"
          />
        </div>
      </section>
    </template>

    <NewProjectWizard v-model:visible="showNew" @submit="onCreate" />
    <SwitchProjectDialog
      v-model:visible="showSwitch"
      :next-title="switchTarget?.title ?? ''"
      :current-title="active?.title ?? null"
      @confirm="confirmSwitch"
    />
  </section>
</template>
