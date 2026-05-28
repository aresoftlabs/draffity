<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { save } from '@tauri-apps/plugin-dialog';
import Dialog from 'primevue/dialog';
import Select from 'primevue/select';
import Button from 'primevue/button';
import { useToast } from 'primevue/usetoast';
import type { ExportFormat, Project } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

const props = defineProps<{
  visible: boolean;
  project: Project | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

const { t } = useI18n();
const { run } = useIpcError();
const toast = useToast();

const format = ref<ExportFormat>('markdown');
const exporting = ref(false);

const formatOptions = computed(() => [
  { value: 'markdown', label: t('export.formatMarkdown'), icon: 'pi pi-file' },
  { value: 'docx', label: t('export.formatDocx'), icon: 'pi pi-file-word' },
  { value: 'epub', label: t('export.formatEpub'), icon: 'pi pi-book' },
]);

const extension = computed(() => {
  switch (format.value) {
    case 'markdown':
      return 'md';
    case 'docx':
      return 'docx';
    case 'epub':
      return 'epub';
    case 'pdf':
      return 'pdf';
  }
});

const filterByFormat = computed(() => {
  switch (format.value) {
    case 'markdown':
      return [{ name: 'Markdown', extensions: ['md'] }];
    case 'docx':
      return [{ name: 'Word', extensions: ['docx'] }];
    case 'epub':
      return [{ name: 'EPUB', extensions: ['epub'] }];
    case 'pdf':
      return [{ name: 'PDF', extensions: ['pdf'] }];
  }
});

watch(
  () => props.visible,
  (v) => {
    if (v) {
      format.value = 'markdown';
      exporting.value = false;
    }
  },
);

function close() {
  emit('update:visible', false);
}

async function onExport() {
  if (!props.project) return;
  const defaultName = sanitize(props.project.title) + '.' + extension.value;
  const target = await save({
    defaultPath: defaultName,
    filters: filterByFormat.value,
    title: t('export.dialogTitle'),
  });
  if (!target) return;

  exporting.value = true;
  const result = await run(t('errors.exportProject'), () =>
    ipc.exportProject({
      projectId: props.project!.id,
      format: format.value,
      outputPath: target,
    }),
  );
  exporting.value = false;

  if (result) {
    toast.add({
      severity: 'success',
      summary: t('export.successTitle'),
      detail: t('export.successDetail', { path: result }),
      life: 6000,
    });
    close();
  }
}

function sanitize(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, '_').trim() || 'manuscript';
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('export.title')"
    :style="{ width: '28rem' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-4">
      <p class="text-sm opacity-70">{{ t('export.subtitle') }}</p>

      <div class="flex flex-col gap-1">
        <label for="export-format" class="text-sm font-medium">
          {{ t('export.format') }}
        </label>
        <Select
          id="export-format"
          v-model="format"
          :options="formatOptions"
          option-label="label"
          option-value="value"
          class="w-full"
        >
          <template #option="{ option }">
            <span class="flex items-center gap-2">
              <i :class="option.icon" />
              <span>{{ option.label }}</span>
            </span>
          </template>
        </Select>
      </div>
    </div>

    <template #footer>
      <Button
        :label="t('actions.cancel')"
        text
        severity="secondary"
        :disabled="exporting"
        @click="close"
      />
      <Button
        :label="t('actions.export')"
        icon="pi pi-download"
        :loading="exporting"
        :disabled="!project || exporting"
        @click="onExport"
      />
    </template>
  </Dialog>
</template>
