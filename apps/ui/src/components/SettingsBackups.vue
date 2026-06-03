<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import type { BackupRecord } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

/**
 * Backups section of Settings (extracted from the Settings god-view, AUD-28):
 * list snapshots, take a manual one, and restore. Self-contained â€” owns its
 * own state and loads on mount.
 */
const { t } = useI18n();
const toast = useToast();
const { run } = useIpcError();

const backups = ref<BackupRecord[]>([]);
const creatingBackup = ref(false);
const restoringId = ref<string | null>(null);

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

onMounted(loadBackups);
</script>

<template>
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
</template>
