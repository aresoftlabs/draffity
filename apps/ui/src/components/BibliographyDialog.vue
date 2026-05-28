<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import { useToast } from 'primevue/usetoast';
import type { Citation } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

const props = defineProps<{
  visible: boolean;
  projectId: string | null;
}>();

const emit = defineEmits<{ 'update:visible': [value: boolean] }>();

const { t } = useI18n();
const { run } = useIpcError();
const toast = useToast();

const entries = ref<Citation[]>([]);
const importing = ref(false);

watch(
  () => [props.visible, props.projectId] as const,
  async ([v, pid]) => {
    if (!v || !pid) return;
    await refresh(pid);
  },
);

async function refresh(projectId: string) {
  const list = await run(t('bibliography.importError'), () => ipc.listCitations(projectId));
  if (list) entries.value = list;
}

async function onImport() {
  if (!props.projectId) return;
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'BibTeX', extensions: ['bib'] }],
    title: t('bibliography.importPickerTitle'),
  });
  if (typeof picked !== 'string') return;

  importing.value = true;
  const bibText = await readTextFile(picked);
  const summary = await run(t('bibliography.importError'), () =>
    ipc.importBibliography({ projectId: props.projectId!, bibText }),
  );
  importing.value = false;
  if (summary) {
    entries.value = summary.imported;
    toast.add({
      severity: 'success',
      summary: t('bibliography.title'),
      detail: t('bibliography.importSuccess', {
        imported: summary.imported.length,
        skipped: summary.skipped,
      }),
      life: 5000,
    });
  }
}

async function onDelete(c: Citation) {
  if (!props.projectId) return;
  if (!confirm(t('bibliography.deleteConfirm'))) return;
  await run(t('bibliography.importError'), () => ipc.deleteCitation(c.id));
  entries.value = entries.value.filter((e) => e.id !== c.id);
}

function close() {
  emit('update:visible', false);
}

function authorOf(c: Citation): string {
  return c.fields.author ?? '';
}
function yearOf(c: Citation): string {
  return c.fields.year ?? '';
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('bibliography.title')"
    :style="{ width: '46rem', maxHeight: '85vh' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-4">
      <p class="text-sm opacity-70">{{ t('bibliography.subtitle') }}</p>
      <div class="flex">
        <Button
          :label="t('bibliography.importButton')"
          icon="pi pi-upload"
          :loading="importing"
          @click="onImport"
        />
      </div>

      <DataTable
        :value="entries"
        size="small"
        striped-rows
        :paginator="entries.length > 10"
        :rows="10"
        scrollable
        scroll-height="50vh"
        :pt="{ wrapper: { class: 'rounded border border-surface-200 dark:border-surface-700' } }"
      >
        <template #empty>
          <div class="p-4 text-center text-sm opacity-60">
            {{ t('bibliography.empty') }}
          </div>
        </template>
        <Column field="key" :header="t('bibliography.key')" sortable />
        <Column field="entryType" :header="t('bibliography.type')" sortable style="width: 8rem" />
        <Column :header="t('bibliography.author')">
          <template #body="{ data }: { data: Citation }">{{ authorOf(data) }}</template>
        </Column>
        <Column :header="t('bibliography.year')" style="width: 5rem">
          <template #body="{ data }: { data: Citation }">{{ yearOf(data) }}</template>
        </Column>
        <Column style="width: 3rem">
          <template #body="{ data }: { data: Citation }">
            <Button
              icon="pi pi-trash"
              text
              severity="danger"
              size="small"
              :aria-label="t('actions.delete')"
              @click="onDelete(data)"
            />
          </template>
        </Column>
      </DataTable>
    </div>

    <template #footer>
      <Button :label="t('actions.back')" text severity="secondary" @click="close" />
    </template>
  </Dialog>
</template>
