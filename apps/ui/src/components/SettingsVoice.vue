<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';
import Select from 'primevue/select';
import ToggleSwitch from 'primevue/toggleswitch';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  ipc,
  type AccelStatus,
  type VoiceModel,
  type VoiceStatus,
  type VoiceVoice,
  type VoiceDownloadProgress,
  type DiskUsageEntry,
  type LanguageGroup,
} from '@/services/ipc';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { useVoiceRecorder } from '@/audio/useVoiceRecorder';
import VoiceRecorderControl from './VoiceRecorderControl.vue';
/**
 * Voice section of Settings (Épica H), extracted from the god-view (AUD-28).
 * Manages the Whisper/Piper binaries and the downloadable model/voice catalog,
 * with live download progress. Self-contained.
 */
const { t } = useI18n();
const toast = useToast();
const voiceSettings = useVoiceSettingsStore();

const testingVoiceId = ref<string | null>(null);
const asrResult = ref<string | null>(null);
const asrMicDenied = ref(false);
const inputDevices = ref<{ label: string; value: string }[]>([]);
const availableModels = ref<LanguageGroup[]>([]);
const catalogError = ref(false);

const voiceStatus = ref<VoiceStatus | null>(null);
const voiceModels = ref<VoiceModel[]>([]);
const voiceVoices = ref<VoiceVoice[]>([]);
const diskUsage = ref<DiskUsageEntry[]>([]);
const downloadPct = ref<Record<string, number>>({});
const importingBinary = ref(false);
const importingPiper = ref(false);
const downloadingBinary = ref<string | null>(null);
let unlistenVoiceProgress: UnlistenFn | null = null;

const accel = ref<AccelStatus | null>(null);

async function loadAccel() {
  try {
    accel.value = await ipc.getAccelStatus();
  } catch {
    accel.value = null;
  }
}

function accelLabel(b: string): string {
  if (b === 'metal') return t('settings.voiceAccelMetal');
  if (b === 'vulkan') return t('settings.voiceAccelVulkan');
  return t('settings.voiceAccelCpu');
}

watch(
  () => voiceSettings.asrModelId,
  async (id) => {
    try {
      await ipc.setSetting('voice.asr.model', id ?? '');
      await loadAccel();
    } catch {
      /* best-effort */
    }
  },
);

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.min(Math.floor(Math.log2(bytes) / 10), units.length - 1);
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

const installedModelIds = computed(
  () => new Set(voiceModels.value.filter((m) => m.installed).map((m) => m.id)),
);

const asrOptions = computed(() =>
  voiceModels.value.filter((m) => m.installed).map((m) => ({ label: m.id, value: m.id })),
);

function isSelectedAsrModelMissing(): boolean {
  const selected = voiceSettings.asrModelId;
  if (!selected) return false;
  return !installedModelIds.value.has(selected);
}

function diskForModel(modelId: string): DiskUsageEntry | undefined {
  return diskUsage.value.find((d) => d.id === modelId);
}

const totalDiskBytes = computed(() => diskUsage.value.reduce((sum, d) => sum + d.bytes, 0));

const installedCount = computed(() => {
  const modelIds = new Set(voiceModels.value.filter((m) => m.installed).map((m) => m.id));
  const voiceInstalled = voiceVoices.value.filter((v) => v.installed).length;
  return modelIds.size + voiceInstalled;
});

async function loadVoice() {
  try {
    voiceStatus.value = await ipc.getVoiceStatus();
    voiceModels.value = await ipc.listVoiceModels();
    voiceVoices.value = await ipc.listVoiceVoices();
    diskUsage.value = await ipc.getDiskUsage();
  } catch (e) {
    voiceStatus.value = null;
    voiceModels.value = [];
    voiceVoices.value = [];
    diskUsage.value = [];
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

async function onDownloadBinary(binaryId: 'whisper' | 'piper') {
  const pctKey = `binary-${binaryId}`;
  downloadPct.value = { ...downloadPct.value, [pctKey]: 0 };
  downloadingBinary.value = binaryId;
  try {
    await ipc.downloadVoiceBinary(binaryId);
    notifyDownloaded();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('[settings] downloadBinary failed:', binaryId, msg);
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: msg,
      life: 8000,
    });
  } finally {
    const rest = { ...downloadPct.value };
    delete rest[pctKey];
    downloadPct.value = rest;
    downloadingBinary.value = null;
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

// Reused across test plays so we don't leak AudioContext instances.
let ttsCtx: AudioContext | null = null;

async function onTestVoice(v: VoiceVoice) {
  testingVoiceId.value = v.id;
  try {
    // Synthesize to PCM16 in-process and play via Web Audio. We deliberately
    // avoid the temp-file + plugin-fs `readFile` round trip: the home dir
    // (~/.draffity) is outside the `fs:default` scope, so reading it throws.
    const audio = await ipc.synthesizeSpeech('Hello, this is a test voice.', v.id);
    if (!ttsCtx) ttsCtx = new AudioContext();
    if (ttsCtx.state === 'suspended') await ttsCtx.resume();
    const len = Math.max(1, audio.samplesPcm16.length);
    const buffer = ttsCtx.createBuffer(1, len, audio.sampleRate || 22050);
    const channel = buffer.getChannelData(0);
    for (let i = 0; i < audio.samplesPcm16.length; i++) channel[i] = audio.samplesPcm16[i] / 32768;
    const source = ttsCtx.createBufferSource();
    source.buffer = buffer;
    source.connect(ttsCtx.destination);
    source.onended = () => {
      testingVoiceId.value = null;
    };
    source.start();
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceTestError'),
      life: 5000,
    });
    testingVoiceId.value = null;
  }
}

/**
 * Enumerate available microphones for the input-device picker. Device labels
 * are only populated by the browser once microphone permission has been
 * granted, so the first call (before any recording) may return unnamed
 * entries — we fall back to a generic numbered label in that case.
 */
async function refreshInputDevices() {
  if (!navigator.mediaDevices?.enumerateDevices) return;
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    const inputs = devices.filter((d) => d.kind === 'audioinput' && d.deviceId);
    inputDevices.value = inputs.map((d, i) => ({
      value: d.deviceId,
      label: d.label || t('settings.voiceInputDeviceUnnamed', { n: String(i + 1) }),
    }));
    // Drop a stale saved selection that no longer matches a present device.
    if (
      voiceSettings.inputDeviceId &&
      !inputs.some((d) => d.deviceId === voiceSettings.inputDeviceId)
    ) {
      voiceSettings.inputDeviceId = null;
    }
  } catch {
    inputDevices.value = [];
  }
}

const recorder = useVoiceRecorder();
const asrPhase = ref<'idle' | 'recording' | 'transcribing'>('idle');

async function onAsrStartRecord() {
  asrMicDenied.value = false;
  asrResult.value = null;
  try {
    await recorder.start(voiceSettings.inputDeviceId);
    asrPhase.value = 'recording';
    // Permiso concedido → ahora hay etiquetas de dispositivos: refrescar el selector.
    void refreshInputDevices();
  } catch {
    asrMicDenied.value = true;
  }
}

async function onAsrStopRecord() {
  if (asrPhase.value !== 'recording') return;
  asrPhase.value = 'transcribing';
  try {
    const rec = await recorder.stop();
    const transcript = await ipc.transcribeAudio(rec.wav);
    asrResult.value = transcript.text;
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceTestError'),
      life: 5000,
    });
  } finally {
    asrPhase.value = 'idle';
  }
}

function onAsrCancel() {
  recorder.cancel();
  asrPhase.value = 'idle';
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

async function loadAvailableModels() {
  catalogError.value = false;
  try {
    const result = await ipc.listAvailableModels();
    availableModels.value = result ?? [];
  } catch {
    catalogError.value = true;
    availableModels.value = [];
  }
}

function onCatalogDownload(item: { id: string; kind: string }) {
  if (item.kind === 'voice') {
    downloadPct.value = { ...downloadPct.value, [item.id]: 0 };
    ipc
      .downloadVoiceVoice(item.id)
      .then(() => {
        notifyDownloaded();
        loadVoice();
        loadAvailableModels();
      })
      .catch(() => notifyVoiceError())
      .finally(() => {
        const rest = { ...downloadPct.value };
        delete rest[item.id];
        downloadPct.value = rest;
      });
  } else {
    // model kind
    downloadPct.value = { ...downloadPct.value, [item.id]: 0 };
    ipc
      .downloadVoiceModel(item.id)
      .then(() => {
        notifyDownloaded();
        loadVoice();
        loadAvailableModels();
      })
      .catch(() => notifyVoiceError())
      .finally(() => {
        const rest = { ...downloadPct.value };
        delete rest[item.id];
        downloadPct.value = rest;
      });
  }
}

onMounted(async () => {
  await loadVoice();
  await loadAccel();
  await loadAvailableModels();
  await refreshInputDevices();
  navigator.mediaDevices?.addEventListener?.('devicechange', refreshInputDevices);
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
  navigator.mediaDevices?.removeEventListener?.('devicechange', refreshInputDevices);
});
</script>

<template>
  <section>
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
      <div class="flex items-center gap-2">
        <span v-if="downloadingBinary === 'whisper'" class="text-xs font-mono opacity-70 mr-2">
          {{ `${downloadPct['binary-whisper'] ?? 0}%` }}
        </span>
        <Button
          v-if="!voiceStatus?.binaryInstalled && downloadingBinary !== 'whisper'"
          :label="t('settings.voiceDownloadBinary')"
          icon="pi pi-download"
          size="small"
          @click="onDownloadBinary('whisper')"
        />
        <Button
          :label="t('settings.voiceImportBinary')"
          size="small"
          text
          :loading="importingBinary"
          @click="onImportBinary"
        />
      </div>
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
          <span v-if="m.installed && diskForModel(m.id)" class="block text-xs opacity-50">
            {{ t('settings.voiceDiskLabel') }}: {{ formatBytes(diskForModel(m.id)!.bytes) }}
          </span>
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

    <!-- ASR model selector -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceModelSelector') }}
      </h3>
      <div class="flex items-center gap-2">
        <Select
          v-model="voiceSettings.asrModelId"
          :options="asrOptions"
          option-label="label"
          option-value="value"
          :placeholder="t('settings.voiceModelSelectorLabel')"
          class="flex-1"
          :aria-label="t('settings.voiceModelSelector')"
        />
        <span
          v-if="isSelectedAsrModelMissing()"
          class="text-xs px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 whitespace-nowrap"
        >
          {{ t('settings.voiceModelNotInstalled') }}
        </span>
      </div>
      <div v-if="accel" class="mt-3 flex items-center gap-3 text-sm" data-test="accel-panel">
        <span class="opacity-70">{{ t('settings.voiceAccel') }}:</span>
        <span class="font-medium">{{ accelLabel(accel.backend) }}</span>
        <template v-if="accel.model">
          <span class="opacity-70">· {{ t('settings.voiceAccelModel') }}:</span>
          <span class="font-mono text-xs">{{ accel.model }}</span>
        </template>
        <Button
          :label="t('settings.voiceAccelRedetect')"
          size="small"
          text
          icon="pi pi-refresh"
          @click="loadAccel"
        />
      </div>
    </div>

    <!-- Microphone (input device) selector -->
    <div class="mt-3">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceInputDevice') }}
      </h3>
      <div class="flex items-center gap-2">
        <Select
          v-model="voiceSettings.inputDeviceId"
          :options="inputDevices"
          option-label="label"
          option-value="value"
          :placeholder="t('settings.voiceInputDeviceDefault')"
          show-clear
          class="flex-1"
          :aria-label="t('settings.voiceInputDevice')"
          :empty-message="t('settings.voiceInputDeviceEmpty')"
        />
        <Button
          icon="pi pi-refresh"
          size="small"
          text
          :aria-label="t('settings.voiceInputDeviceRefresh')"
          @click="refreshInputDevices"
        />
      </div>
      <p class="text-xs opacity-50 mt-1">{{ t('settings.voiceInputDeviceHint') }}</p>
    </div>

    <!-- Auto-stop on silence toggle -->
    <div class="mt-3 flex items-center gap-2">
      <ToggleSwitch
        v-model="voiceSettings.autoStopOnSilence"
        input-id="auto-stop-silence"
        :aria-label="t('settings.voiceAutoStopSilence')"
      />
      <label for="auto-stop-silence" class="text-sm">{{
        t('settings.voiceAutoStopSilence')
      }}</label>
    </div>
    <p class="text-xs opacity-50 mt-1">{{ t('settings.voiceAutoStopSilenceHint') }}</p>

    <!-- ASR test recorder -->
    <div class="mt-3">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceAsrTestLabel') }}
      </h3>
      <div v-if="asrPhase === 'idle'">
        <Button
          :label="t('settings.voiceAsrTestRecord')"
          icon="pi pi-microphone"
          size="small"
          text
          @click="onAsrStartRecord"
        />
      </div>
      <VoiceRecorderControl
        v-else
        :state="asrPhase === 'recording' ? 'recording' : 'transcribing'"
        :waveform="recorder.waveform.value"
        :elapsed-ms="recorder.elapsedMs.value"
        :is-silent="recorder.isSilent.value"
        :progress="null"
        @stop="onAsrStopRecord"
        @cancel="onAsrCancel"
      />
      <p v-if="asrMicDenied" class="text-xs text-amber-500 mt-1">
        {{ t('settings.voiceAsrTestMicDenied') }}
      </p>
      <Textarea
        v-if="asrResult !== null"
        :value="asrResult"
        readonly
        rows="2"
        class="w-full mt-2 text-xs"
        :placeholder="t('settings.voiceAsrTestResult')"
      />
    </div>

    <!-- Disk usage summary -->
    <div class="mt-3 mb-1">
      <template v-if="installedCount > 0">
        <p class="text-xs opacity-70">
          {{
            t('settings.voiceDiskUsage', {
              count: String(installedCount),
              size: formatBytes(totalDiskBytes),
            })
          }}
        </p>
      </template>
      <p v-else class="text-xs opacity-50 italic">
        {{ t('settings.voiceDiskNone') }}
      </p>
    </div>

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
      <div class="flex items-center gap-2">
        <span v-if="downloadingBinary === 'piper'" class="text-xs font-mono opacity-70 mr-2">
          {{ `${downloadPct['binary-piper'] ?? 0}%` }}
        </span>
        <Button
          v-if="!voiceStatus?.piperInstalled && downloadingBinary !== 'piper'"
          :label="t('settings.voiceDownloadBinary')"
          icon="pi pi-download"
          size="small"
          @click="onDownloadBinary('piper')"
        />
        <Button
          :label="t('settings.voiceImportPiper')"
          size="small"
          text
          :loading="importingPiper"
          @click="onImportPiper"
        />
      </div>
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
        <div class="shrink-0 flex items-center gap-2">
          <template v-if="v.installed">
            <Button
              :label="
                testingVoiceId === v.id
                  ? t('settings.voiceTestPlaying')
                  : t('settings.voiceTestPlay')
              "
              icon="pi pi-play"
              size="small"
              text
              :loading="testingVoiceId === v.id"
              :disabled="testingVoiceId === v.id"
              @click.stop="onTestVoice(v)"
            />
            <span class="text-xs text-green-600 dark:text-green-400">
              {{ t('settings.voiceInstalled') }}
            </span>
          </template>
          <template v-else>
            <span v-if="downloadPct[v.id] !== undefined" class="text-xs font-mono opacity-70">
              {{ `${downloadPct[v.id]}%` }}
            </span>
            <Button
              v-else
              :label="t('settings.voiceModelDownload')"
              icon="pi pi-download"
              size="small"
              text
              @click="onDownloadVoice(v)"
            />
          </template>
        </div>
      </li>
    </ul>

    <!-- Voice catalog -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceCatalog') }}
      </h3>
      <div v-if="catalogError" class="flex items-center gap-2">
        <p class="text-xs opacity-60">{{ t('settings.voiceCatalogEmpty') }}</p>
        <Button
          :label="t('settings.voiceCatalogRetry')"
          icon="pi pi-refresh"
          size="small"
          text
          @click="loadAvailableModels"
        />
      </div>
      <div v-else-if="availableModels.length === 0">
        <p class="text-xs opacity-50 italic">{{ t('settings.voiceCatalogEmpty') }}</p>
      </div>
      <div v-else class="divide-y divide-surface-200 dark:divide-surface-700">
        <div v-for="group in availableModels" :key="group.lang" class="py-2">
          <h4 class="text-xs font-semibold opacity-70 mb-1">
            {{ t('settings.voiceCatalogByLang', { lang: group.lang.toUpperCase() }) }}
          </h4>
          <div
            v-for="item in group.items"
            :key="item.id"
            class="flex items-center justify-between gap-2 py-1 text-xs"
          >
            <span>{{ item.name }}</span>
            <span class="opacity-50">{{
              t('settings.voiceCatalogSize', { size: String(item.sizeMb) })
            }}</span>
            <div class="shrink-0 flex items-center gap-2">
              <template v-if="downloadPct[item.id] !== undefined">
                <span class="font-mono opacity-70">{{ downloadPct[item.id] }}%</span>
              </template>
              <span v-else-if="item.installed" class="text-green-600 dark:text-green-400">
                {{ t('settings.voiceInstalled') }}
              </span>
              <Button
                v-else
                :label="t('settings.voiceModelDownload')"
                icon="pi pi-download"
                size="small"
                text
                @click="onCatalogDownload(item)"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
