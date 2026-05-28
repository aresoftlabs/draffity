<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import SelectButton from 'primevue/selectbutton';
import Slider from 'primevue/slider';
import ToggleSwitch from 'primevue/toggleswitch';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';
import { useToast } from 'primevue/usetoast';
import { useUiStore } from '@/stores/ui';
import { useEditorSettings, type EditorFont } from '@/composables/useEditorSettings';
import { useIpcError } from '@/composables/useIpcError';
import { ipc } from '@/services/ipc';
import type { BackupRecord, WritingStats } from '@draffity/shared-types';

const { t, locale } = useI18n();
const ui = useUiStore();
const { autosaveMs, font, customCss } = useEditorSettings();

const themeOptions = computed(() => [
  { label: t('settings.themeLight'), value: 'light' },
  { label: t('settings.themeDark'), value: 'dark' },
  { label: t('settings.themeSystem'), value: 'system' },
]);

const localeOptions = [
  { label: 'Español', value: 'es' },
  { label: 'English', value: 'en' },
];

const fontOptions = computed(() => [
  { label: t('settings.fontSerif'), value: 'serif' as EditorFont },
  { label: t('settings.fontSans'), value: 'sans' as EditorFont },
  { label: t('settings.fontMono'), value: 'mono' as EditorFont },
]);

const themeModel = computed({
  get: () => ui.theme,
  set: (v) => ui.setTheme(v as 'light' | 'dark' | 'system'),
});

const localeModel = computed({
  get: () => locale.value,
  set: (v: string) => ui.setLocale(v as 'es' | 'en'),
});

const stats = ref<WritingStats | null>(null);
const backups = ref<BackupRecord[]>([]);
const creatingBackup = ref(false);
const restoringId = ref<string | null>(null);
const { run } = useIpcError();
const toast = useToast();

onMounted(async () => {
  try {
    stats.value = await ipc.getWritingStats();
  } catch {
    stats.value = null;
  }
  await loadBackups();
});

async function loadBackups() {
  const list = await run(t('settings.backupsError'), () => ipc.listBackups());
  if (list) backups.value = list;
}

async function onCreateBackup() {
  creatingBackup.value = true;
  const rec = await run(t('settings.backupsError'), () => ipc.createManualBackup());
  creatingBackup.value = false;
  if (rec) {
    backups.value = [rec, ...backups.value];
    toast.add({
      severity: 'success',
      summary: t('settings.backupsTitle'),
      detail: t('settings.backupCreated'),
      life: 3000,
    });
  }
}

async function onRestore(b: BackupRecord) {
  if (!confirm(t('settings.restoreConfirm'))) return;
  restoringId.value = b.id;
  await run(t('settings.backupsError'), () => ipc.restoreBackup(b.id));
  restoringId.value = null;
  toast.add({
    severity: 'success',
    summary: t('settings.backupsTitle'),
    detail: t('settings.restoreSuccess'),
    life: 6000,
  });
  await loadBackups();
}

function formatDate(ms: number): string {
  return new Date(ms).toLocaleString();
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function kindLabel(kind: BackupRecord['kind']): string {
  return t(`settings.backupKind.${kind}`);
}
</script>

<template>
  <section class="flex-1 p-8 max-w-2xl w-full mx-auto">
    <h1 class="text-2xl font-serif font-bold mb-8">{{ t('settings.title') }}</h1>

    <div class="space-y-8">
      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.theme') }}
        </h2>
        <SelectButton
          v-model="themeModel"
          :options="themeOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.language') }}
        </h2>
        <SelectButton
          v-model="localeModel"
          :options="localeOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.editorFont') }}
        </h2>
        <SelectButton
          v-model="font"
          :options="fontOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.autosave') }}
          <span class="font-mono opacity-60">{{ autosaveMs }} ms</span>
        </h2>
        <Slider v-model="autosaveMs" :min="200" :max="3000" :step="100" />
        <p class="text-xs opacity-60 mt-2">{{ t('settings.autosaveHint') }}</p>
      </section>

      <section class="flex items-center justify-between gap-4">
        <div>
          <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
            {{ t('settings.typewriter') }}
          </h2>
          <p class="text-xs opacity-60 mt-1">{{ t('settings.typewriterHint') }}</p>
        </div>
        <ToggleSwitch v-model="ui.typewriterMode" />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
          {{ t('settings.customCss') }}
        </h2>
        <p class="text-xs opacity-60 mb-2">{{ t('settings.customCssHint') }}</p>
        <Textarea
          v-model="customCss"
          rows="6"
          class="w-full font-mono text-xs"
          :placeholder="t('settings.customCssPlaceholder')"
          spellcheck="false"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.writingStats') }}
        </h2>
        <dl v-if="stats" class="text-sm space-y-1">
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.currentStreak') }}</dt>
            <dd class="font-mono">{{ stats.currentStreak }}</dd>
          </div>
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.longestStreak') }}</dt>
            <dd class="font-mono">{{ stats.longestStreak }}</dd>
          </div>
          <div v-if="stats.lastWritingDate" class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.lastWritingDate') }}</dt>
            <dd class="font-mono">{{ stats.lastWritingDate }}</dd>
          </div>
        </dl>
        <p v-else class="text-xs opacity-60">…</p>
      </section>

      <section>
        <div class="flex items-center justify-between mb-3 gap-3">
          <div>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
              {{ t('settings.backupsTitle') }}
            </h2>
            <p class="text-xs opacity-60 mt-1">{{ t('settings.backupsHint') }}</p>
          </div>
          <Button
            :label="t('settings.backupNow')"
            icon="pi pi-database"
            size="small"
            :loading="creatingBackup"
            @click="onCreateBackup"
          />
        </div>
        <div
          v-if="backups.length === 0"
          class="text-xs opacity-60 p-3 rounded border border-surface-200 dark:border-surface-700"
        >
          {{ t('settings.backupsEmpty') }}
        </div>
        <ul
          v-else
          class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
        >
          <li
            v-for="b in backups"
            :key="b.id"
            class="flex items-center justify-between gap-3 p-3 text-sm"
          >
            <div class="flex flex-col min-w-0">
              <span class="font-mono text-xs truncate">{{ b.id }}</span>
              <span class="text-xs opacity-60">
                {{ kindLabel(b.kind) }} · {{ formatDate(b.createdAt) }} ·
                {{ formatSize(b.sizeBytes) }}
              </span>
            </div>
            <Button
              :label="t('settings.restore')"
              size="small"
              text
              :loading="restoringId === b.id"
              @click="onRestore(b)"
            />
          </li>
        </ul>
      </section>
    </div>
  </section>
</template>
