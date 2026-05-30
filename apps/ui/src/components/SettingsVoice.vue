<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  ipc,
  type VoiceModel,
  type VoiceStatus,
  type VoiceVoice,
  type VoiceDownloadProgress,
} from '@/services/ipc';
import { useCapability } from '@/composables/useCapability';

/**
 * Voice section of Settings (Épica H), extracted from the god-view (AUD-28).
 * Premium-gated (`voice_to_text`): manage the Whisper/Piper binaries and the
 * downloadable model/voice catalog, with live download progress. Self-contained.
 */
const { t } = useI18n();
const toast = useToast();
const voiceEnabled = useCapability('voice_to_text');

const voiceStatus = ref<VoiceStatus | null>(null);
const voiceModels = ref<VoiceModel[]>([]);
const voiceVoices = ref<VoiceVoice[]>([]);
const downloadPct = ref<Record<string, number>>({});
const importingBinary = ref(false);
const importingPiper = ref(false);
let unlistenVoiceProgress: UnlistenFn | null = null;

async function loadVoice() {
  try {
    voiceStatus.value = await ipc.getVoiceStatus();
    voiceModels.value = await ipc.listVoiceModels();
    voiceVoices.value = await ipc.listVoiceVoices();
  } catch (e) {
    voiceStatus.value = null;
    voiceModels.value = [];
    voiceVoices.value = [];
    console.error('[settings]', 'voice', e);
  }
}

function notifyDownloaded() {
  toast.add({
    severity: 'success',
    summary: t('settings.voiceTitle'),
    detail: t('settings.voiceModelDownloaded'),
    life: 3000,
  });
}

function notifyVoiceError() {
  toast.add({
    severity: 'error',
    summary: t('settings.voiceTitle'),
    detail: t('settings.voiceModelError'),
    life: 5000,
  });
}

function notifyBinaryImported() {
  toast.add({
    severity: 'success',
    summary: t('settings.voiceTitle'),
    detail: t('settings.voiceBinaryImported'),
    life: 3000,
  });
}

async function onDownloadVoice(v: VoiceVoice) {
  downloadPct.value = { ...downloadPct.value, [v.id]: 0 };
  try {
    await ipc.downloadVoiceVoice(v.id);
    notifyDownloaded();
  } catch {
    notifyVoiceError();
  } finally {
    const rest = { ...downloadPct.value };
    delete rest[v.id];
    downloadPct.value = rest;
    await loadVoice();
  }
}

async function onImportPiper() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t('settings.voiceImportPiper'),
  });
  if (typeof picked !== 'string') return;
  importingPiper.value = true;
  try {
    await ipc.importPiperBinary(picked);
    notifyBinaryImported();
  } catch {
    notifyVoiceError();
  } finally {
    importingPiper.value = false;
    await loadVoice();
  }
}

async function onDownloadModel(m: VoiceModel) {
  downloadPct.value = { ...downloadPct.value, [m.id]: 0 };
  try {
    await ipc.downloadVoiceModel(m.id);
    notifyDownloaded();
  } catch {
    notifyVoiceError();
  } finally {
    const rest = { ...downloadPct.value };
    delete rest[m.id];
    downloadPct.value = rest;
    await loadVoice();
  }
}

async function onDeleteModel(m: VoiceModel) {
  try {
    await ipc.deleteVoiceModel(m.id);
  } catch {
    // best-effort
  }
  await loadVoice();
}

async function onImportBinary() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t('settings.voiceImportBinary'),
  });
  if (typeof picked !== 'string') return;
  importingBinary.value = true;
  try {
    await ipc.importVoiceBinary(picked);
    notifyBinaryImported();
  } catch {
    notifyVoiceError();
  } finally {
    importingBinary.value = false;
    await loadVoice();
  }
}

onMounted(async () => {
  await loadVoice();
  unlistenVoiceProgress = await listen<VoiceDownloadProgress>('voice.download.progress', (e) => {
    const p = e.payload;
    if (p.total && p.total > 0) {
      downloadPct.value = {
        ...downloadPct.value,
        [p.modelId]: Math.round((p.downloaded / p.total) * 100),
      };
    }
  });
});

onBeforeUnmount(() => {
  unlistenVoiceProgress?.();
});
</script>

<template>
  <section v-if="voiceEnabled">
    <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
      {{ t('settings.voiceTitle') }}
    </h2>
    <p class="text-xs opacity-60 mb-2">{{ t('settings.voiceHint') }}</p>

    <!-- Whisper binary -->
    <div
      class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
    >
      <span>
        <i
          :class="
            voiceStatus?.binaryInstalled
              ? 'pi pi-check-circle text-green-500'
              : 'pi pi-exclamation-circle text-amber-500'
          "
          class="mr-1"
        />
        {{
          voiceStatus?.binaryInstalled
            ? t('settings.voiceBinaryInstalled')
            : t('settings.voiceBinaryMissing')
        }}
      </span>
      <Button
        :label="t('settings.voiceImportBinary')"
        size="small"
        text
        :loading="importingBinary"
        @click="onImportBinary"
      />
    </div>

    <!-- Models -->
    <ul
      class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
    >
      <li
        v-for="m in voiceModels"
        :key="m.id"
        class="flex items-center justify-between gap-3 p-3 text-sm"
      >
        <div class="min-w-0">
          <span class="font-medium">{{ m.id }}</span>
          <span
            v-if="m.recommended"
            class="ml-2 text-xs px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-300"
          >
            {{ t('settings.voiceRecommended') }}
          </span>
          <span class="block text-xs opacity-60">{{ `${m.sizeMb} MB` }}</span>
        </div>
        <div class="shrink-0">
          <span v-if="downloadPct[m.id] !== undefined" class="text-xs font-mono opacity-70">
            {{ `${downloadPct[m.id]}%` }}
          </span>
          <Button
            v-else-if="m.installed"
            :label="t('settings.voiceModelDelete')"
            size="small"
            text
            severity="danger"
            @click="onDeleteModel(m)"
          />
          <Button
            v-else
            :label="t('settings.voiceModelDownload')"
            icon="pi pi-download"
            size="small"
            text
            @click="onDownloadModel(m)"
          />
        </div>
      </li>
    </ul>

    <!-- Read-aloud: Piper binary + voices -->
    <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mt-4 mb-2">
      {{ t('settings.voiceReadAloud') }}
    </h3>
    <div
      class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
    >
      <span>
        <i
          :class="
            voiceStatus?.piperInstalled
              ? 'pi pi-check-circle text-green-500'
              : 'pi pi-exclamation-circle text-amber-500'
          "
          class="mr-1"
        />
        {{
          voiceStatus?.piperInstalled
            ? t('settings.voicePiperInstalled')
            : t('settings.voicePiperMissing')
        }}
      </span>
      <Button
        :label="t('settings.voiceImportPiper')"
        size="small"
        text
        :loading="importingPiper"
        @click="onImportPiper"
      />
    </div>
    <ul
      class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
    >
      <li
        v-for="v in voiceVoices"
        :key="v.id"
        class="flex items-center justify-between gap-3 p-3 text-sm"
      >
        <div class="min-w-0">
          <span class="font-medium">{{ v.name }}</span>
          <span
            v-if="v.recommended"
            class="ml-2 text-xs px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-300"
          >
            {{ t('settings.voiceRecommended') }}
          </span>
          <span class="block text-xs opacity-60">{{ `${v.sizeMb} MB` }}</span>
        </div>
        <div class="shrink-0">
          <span v-if="downloadPct[v.id] !== undefined" class="text-xs font-mono opacity-70">
            {{ `${downloadPct[v.id]}%` }}
          </span>
          <span v-else-if="v.installed" class="text-xs text-green-600 dark:text-green-400">
            {{ t('settings.voiceInstalled') }}
          </span>
          <Button
            v-else
            :label="t('settings.voiceModelDownload')"
            icon="pi pi-download"
            size="small"
            text
            @click="onDownloadVoice(v)"
          />
        </div>
      </li>
    </ul>
  </section>
  <p v-else class="text-sm opacity-60">{{ t('capability.unavailable') }}</p>
</template>
