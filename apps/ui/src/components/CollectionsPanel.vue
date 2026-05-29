<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import type { DocNode } from '@draffity/shared-types';
import { useIpcError } from '@/composables/useIpcError';
import { ipc, type Collection } from '@/services/ipc';
import CollectionEditorDialog from '@/components/CollectionEditorDialog.vue';

const props = defineProps<{
  projectId: string;
  currentDocId: string | null;
  readOnly?: boolean;
}>();
const emit = defineEmits<{ select: [string] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();

const collections = ref<Collection[]>([]);
const expandedId = ref<string | null>(null);
const resolved = ref<DocNode[]>([]);
const editorVisible = ref(false);
const editing = ref<Collection | null>(null);

async function load() {
  collections.value =
    (await ipcRun(t('collections.error'), () => ipc.listCollections(props.projectId))) ?? [];
}

async function resolve(id: string) {
  resolved.value = (await ipcRun(t('collections.error'), () => ipc.resolveCollection(id))) ?? [];
}

async function toggle(c: Collection) {
  if (expandedId.value === c.id) {
    expandedId.value = null;
    resolved.value = [];
    return;
  }
  expandedId.value = c.id;
  await resolve(c.id);
}

function onNew() {
  editing.value = null;
  editorVisible.value = true;
}

function onEdit(c: Collection) {
  editing.value = c;
  editorVisible.value = true;
}

async function onDelete(c: Collection) {
  if (!confirm(t('collections.confirmDelete', { name: c.name }))) return;
  await ipcRun(t('collections.error'), () => ipc.deleteCollection(c.id));
  if (expandedId.value === c.id) {
    expandedId.value = null;
    resolved.value = [];
  }
  await load();
}

async function onSaved() {
  await load();
  if (expandedId.value) await resolve(expandedId.value);
}

const alreadyMember = (docId: string | null) =>
  !!docId && resolved.value.some((d) => d.id === docId);

async function addCurrent(c: Collection) {
  if (!props.currentDocId || alreadyMember(props.currentDocId)) return;
  const ids = [...resolved.value.map((d) => d.id), props.currentDocId];
  await ipcRun(t('collections.error'), () => ipc.setCollectionMembers(c.id, ids));
  await resolve(c.id);
}

async function removeMember(c: Collection, docId: string) {
  const ids = resolved.value.map((d) => d.id).filter((id) => id !== docId);
  await ipcRun(t('collections.error'), () => ipc.setCollectionMembers(c.id, ids));
  await resolve(c.id);
}

watch(
  () => props.projectId,
  () => {
    expandedId.value = null;
    resolved.value = [];
    void load();
  },
  { immediate: true },
);
</script>

<template>
  <section class="border-t border-surface-200 dark:border-surface-700 flex flex-col min-h-0">
    <header class="flex items-center justify-between px-3 py-2">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-70">
        {{ t('collections.title') }}
      </h3>
      <Button
        icon="pi pi-plus"
        text
        size="small"
        :aria-label="t('collections.newTitle')"
        @click="onNew"
      />
    </header>

    <div class="overflow-auto px-1 pb-2 max-h-64">
      <p v-if="collections.length === 0" class="text-xs opacity-50 px-2 py-1">
        {{ t('collections.empty') }}
      </p>
      <div v-for="c in collections" :key="c.id">
        <div
          class="group flex items-center gap-1 px-2 py-1 rounded hover:bg-surface-100 dark:hover:bg-surface-800"
        >
          <button
            class="flex items-center gap-2 flex-1 min-w-0 text-left text-sm"
            @click="toggle(c)"
          >
            <i
              :class="c.kind === 'smart' ? 'pi pi-filter' : 'pi pi-folder'"
              class="text-xs opacity-70"
            />
            <span class="truncate">{{ c.name }}</span>
          </button>
          <Button
            icon="pi pi-pencil"
            text
            size="small"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('collections.editTitle')"
            @click="onEdit(c)"
          />
          <Button
            icon="pi pi-trash"
            text
            size="small"
            severity="danger"
            class="opacity-0 group-hover:opacity-100"
            :aria-label="t('collections.delete')"
            @click="onDelete(c)"
          />
        </div>

        <div v-if="expandedId === c.id" class="pl-6 pr-2 pb-1">
          <p v-if="resolved.length === 0" class="text-xs opacity-50 py-1">
            {{ t('collections.noDocs') }}
          </p>
          <div
            v-for="d in resolved"
            :key="d.id"
            class="group/doc flex items-center gap-1 text-sm py-0.5"
          >
            <button
              class="flex-1 min-w-0 text-left truncate hover:underline"
              @click="emit('select', d.id)"
            >
              {{ d.title }}
            </button>
            <Button
              v-if="c.kind === 'manual' && !readOnly"
              icon="pi pi-times"
              text
              size="small"
              class="opacity-0 group-hover/doc:opacity-100"
              :aria-label="t('collections.remove')"
              @click="removeMember(c, d.id)"
            />
          </div>
          <Button
            v-if="c.kind === 'manual' && !readOnly"
            :label="t('collections.addCurrent')"
            icon="pi pi-plus"
            text
            size="small"
            :disabled="!currentDocId || alreadyMember(currentDocId)"
            @click="addCurrent(c)"
          />
        </div>
      </div>
    </div>

    <CollectionEditorDialog
      v-model:visible="editorVisible"
      :project-id="projectId"
      :collection="editing"
      @saved="onSaved"
    />
  </section>
</template>
