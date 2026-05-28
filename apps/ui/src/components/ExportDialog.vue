<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { open, save } from '@tauri-apps/plugin-dialog';
import Dialog from 'primevue/dialog';
import Select from 'primevue/select';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Fieldset from 'primevue/fieldset';
import Checkbox from 'primevue/checkbox';
import { useToast } from 'primevue/usetoast';
import type {
  ExportConfig,
  ExportFormat,
  PageSize,
  Project,
  SceneSeparator,
} from '@draffity/shared-types';
import { DEFAULT_EXPORT_CONFIG } from '@draffity/shared-types';
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
const config = ref<ExportConfig>(cloneDefault());
const saveAsDefault = ref(true);

/** Selected scene-separator kind. Kept in a flat string for the Select; on
 * export we marshal back into the tagged union understood by Rust. */
const sceneKind = ref<'stars' | 'dashes' | 'blank' | 'custom'>('stars');
const sceneCustom = ref('');

/** Selected page-size key. Custom is a special path with width/height fields,
 * but for the dialog we only expose the three standard sizes — Custom is
 * reserved for power users via stored config (or a future advanced toggle). */
const pageKey = ref<'a4' | 'letter' | 'legal'>('a4');

const formatOptions = computed(() => [
  { value: 'markdown', label: t('export.formatMarkdown'), icon: 'pi pi-file' },
  { value: 'docx', label: t('export.formatDocx'), icon: 'pi pi-file-word' },
  { value: 'epub', label: t('export.formatEpub'), icon: 'pi pi-book' },
]);

const fontOptions = computed(() => [
  { value: '', label: t('export.fontDefault') },
  { value: 'serif', label: t('export.fontSerif') },
  { value: 'sans-serif', label: t('export.fontSans') },
  { value: 'monospace', label: t('export.fontMono') },
]);

const pageOptions = computed(() => [
  { value: 'a4', label: t('export.pageA4') },
  { value: 'letter', label: t('export.pageLetter') },
  { value: 'legal', label: t('export.pageLegal') },
]);

const sceneOptions = computed(() => [
  { value: 'stars', label: t('export.sepStars') },
  { value: 'dashes', label: t('export.sepDashes') },
  { value: 'blank', label: t('export.sepBlank') },
  { value: 'custom', label: t('export.sepCustom') },
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
  () => [props.visible, props.project?.id] as const,
  async ([v, pid]) => {
    if (!v || !pid) return;
    exporting.value = false;
    format.value = 'markdown';
    const loaded = await run(t('errors.exportProject'), () => ipc.getExportConfig(pid));
    config.value = loaded ?? cloneDefault();
    syncFromConfig(config.value);
  },
);

function close() {
  emit('update:visible', false);
}

function cloneDefault(): ExportConfig {
  return JSON.parse(JSON.stringify(DEFAULT_EXPORT_CONFIG)) as ExportConfig;
}

function syncFromConfig(cfg: ExportConfig) {
  if (typeof cfg.pageSize === 'string') {
    pageKey.value = cfg.pageSize;
  } else {
    // Custom: keep UI on A4 but preserve the custom dims in `config`.
    pageKey.value = 'a4';
  }
  sceneKind.value = cfg.sceneSeparator.kind;
  sceneCustom.value = cfg.sceneSeparator.kind === 'custom' ? cfg.sceneSeparator.value : '';
}

function buildSceneSeparator(): SceneSeparator {
  if (sceneKind.value === 'custom') {
    return { kind: 'custom', value: sceneCustom.value };
  }
  return { kind: sceneKind.value };
}

function buildPageSize(): PageSize {
  // Custom dims survive across edits even though the dropdown only shows
  // the three presets — if the stored config had custom, switching presets
  // here overrides it intentionally.
  return pageKey.value;
}

function buildConfig(): ExportConfig {
  return {
    ...config.value,
    pageSize: buildPageSize(),
    sceneSeparator: buildSceneSeparator(),
  };
}

async function onExport() {
  if (!props.project) return;
  const defaultName = sanitize(displayTitle()) + '.' + extension.value;
  const target = await save({
    defaultPath: defaultName,
    filters: filterByFormat.value,
    title: t('export.dialogTitle'),
  });
  if (!target) return;

  exporting.value = true;
  const built = buildConfig();
  const result = await run(t('errors.exportProject'), () =>
    ipc.exportProject({
      projectId: props.project!.id,
      format: format.value,
      outputPath: target,
      config: built,
    }),
  );

  if (result && saveAsDefault.value) {
    await run(t('errors.exportProject'), () =>
      ipc.setExportConfig({ projectId: props.project!.id, config: built }),
    );
  }

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

function displayTitle(): string {
  const override = config.value.titleOverride?.trim();
  if (override) return override;
  return props.project?.title ?? 'manuscript';
}

function sanitize(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, '_').trim() || 'manuscript';
}

async function pickCoverImage() {
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp'] }],
    title: t('export.coverPickerTitle'),
  });
  if (typeof picked === 'string') {
    config.value.coverImagePath = picked;
  }
}

function clearCoverImage() {
  config.value.coverImagePath = null;
}

function coverFilename(path: string | null | undefined): string {
  if (!path) return '';
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] ?? '';
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('export.title')"
    :style="{ width: '38rem', maxHeight: '90vh' }"
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

      <Fieldset :legend="t('export.sectionContent')" :toggleable="true">
        <div class="flex flex-col gap-3">
          <div class="flex flex-col gap-1">
            <label for="export-title" class="text-sm font-medium">
              {{ t('export.titleOverride') }}
            </label>
            <InputText
              id="export-title"
              v-model="config.titleOverride"
              :placeholder="props.project?.title ?? ''"
            />
            <span class="text-xs opacity-60">{{ t('export.titleOverrideHint') }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <label for="export-author" class="text-sm font-medium">
              {{ t('export.author') }}
            </label>
            <InputText id="export-author" v-model="config.author" />
          </div>
        </div>
      </Fieldset>

      <Fieldset :legend="t('export.sectionAppearance')" :toggleable="true" :collapsed="true">
        <div class="flex flex-col gap-3">
          <div class="flex flex-col gap-1">
            <label for="export-font" class="text-sm font-medium">
              {{ t('export.fontFamily') }}
            </label>
            <Select
              id="export-font"
              v-model="config.fontFamily"
              :options="fontOptions"
              option-label="label"
              option-value="value"
              class="w-full"
            />
          </div>
          <div class="flex flex-col gap-1">
            <label for="export-page" class="text-sm font-medium">
              {{ t('export.pageSize') }}
            </label>
            <Select
              id="export-page"
              v-model="pageKey"
              :options="pageOptions"
              option-label="label"
              option-value="value"
              class="w-full"
            />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-sm font-medium">{{ t('export.margins') }}</label>
            <div class="grid grid-cols-4 gap-2">
              <div class="flex flex-col gap-1">
                <label for="m-top" class="text-xs opacity-70">{{ t('export.marginTop') }}</label>
                <InputNumber
                  id="m-top"
                  v-model="config.margins.topMm"
                  :min="0"
                  :max="100"
                  size="small"
                />
              </div>
              <div class="flex flex-col gap-1">
                <label for="m-right" class="text-xs opacity-70">{{
                  t('export.marginRight')
                }}</label>
                <InputNumber
                  id="m-right"
                  v-model="config.margins.rightMm"
                  :min="0"
                  :max="100"
                  size="small"
                />
              </div>
              <div class="flex flex-col gap-1">
                <label for="m-bottom" class="text-xs opacity-70">{{
                  t('export.marginBottom')
                }}</label>
                <InputNumber
                  id="m-bottom"
                  v-model="config.margins.bottomMm"
                  :min="0"
                  :max="100"
                  size="small"
                />
              </div>
              <div class="flex flex-col gap-1">
                <label for="m-left" class="text-xs opacity-70">{{ t('export.marginLeft') }}</label>
                <InputNumber
                  id="m-left"
                  v-model="config.margins.leftMm"
                  :min="0"
                  :max="100"
                  size="small"
                />
              </div>
            </div>
          </div>
        </div>
      </Fieldset>

      <Fieldset v-if="format === 'epub'" :legend="t('export.sectionEpub')" :toggleable="true">
        <div class="flex flex-col gap-2">
          <label class="text-sm font-medium">{{ t('export.coverImage') }}</label>
          <div class="flex items-center gap-2">
            <Button
              :label="t('export.coverPickButton')"
              icon="pi pi-image"
              severity="secondary"
              outlined
              size="small"
              @click="pickCoverImage"
            />
            <span v-if="config.coverImagePath" class="flex items-center gap-1 text-sm">
              <span>{{ coverFilename(config.coverImagePath) }}</span>
              <Button
                :label="t('export.coverClear')"
                text
                severity="secondary"
                size="small"
                @click="clearCoverImage"
              />
            </span>
          </div>
        </div>
      </Fieldset>

      <Fieldset :legend="t('export.sectionLayout')" :toggleable="true" :collapsed="true">
        <div class="flex flex-col gap-3">
          <div class="flex items-center gap-2">
            <Checkbox v-model="config.includeTitlePage" input-id="opt-title-page" :binary="true" />
            <label for="opt-title-page" class="text-sm">{{ t('export.includeTitlePage') }}</label>
          </div>
          <div class="flex items-center gap-2">
            <Checkbox v-model="config.includeToc" input-id="opt-toc" :binary="true" />
            <label for="opt-toc" class="text-sm">{{ t('export.includeToc') }}</label>
          </div>
          <div class="flex items-center gap-2">
            <Checkbox v-model="config.includeCodex" input-id="opt-codex" :binary="true" />
            <label for="opt-codex" class="text-sm">{{ t('export.includeCodex') }}</label>
          </div>
          <div class="flex flex-col gap-1">
            <label for="export-scene" class="text-sm font-medium">
              {{ t('export.sceneSeparator') }}
            </label>
            <Select
              id="export-scene"
              v-model="sceneKind"
              :options="sceneOptions"
              option-label="label"
              option-value="value"
              class="w-full"
            />
            <InputText
              v-if="sceneKind === 'custom'"
              v-model="sceneCustom"
              :placeholder="t('export.sepCustomPlaceholder')"
              class="mt-1"
            />
          </div>
        </div>
      </Fieldset>
    </div>

    <template #footer>
      <div class="flex w-full items-center justify-between">
        <div class="flex items-center gap-2">
          <Checkbox v-model="saveAsDefault" input-id="opt-save-default" :binary="true" />
          <label for="opt-save-default" class="text-sm">{{ t('export.saveAsDefault') }}</label>
        </div>
        <div class="flex gap-2">
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
        </div>
      </div>
    </template>
  </Dialog>
</template>
