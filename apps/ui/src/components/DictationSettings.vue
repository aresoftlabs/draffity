<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';
import Select from 'primevue/select';
import ToggleSwitch from 'primevue/toggleswitch';
import { open } from '@tauri-apps/plugin-dialog';
import { ipc, type AccelStatus, type VoiceModel, type VoiceStatus } from '@/services/ipc';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { useVoiceRecorder } from '@/audio/useVoiceRecorder';
import VoiceRecorderControl from './VoiceRecorderControl.vue';

/**
 * "Dictado" block (Whisper/ASR): binary status, model selector, model
 * management, microphone picker, auto-stop toggle and the ASR test recorder.
 * Owns its own model/accel/device state; emits `reload` so the container
 * refetches the shared voice status after a binary download/import.
 */
defineOptions({ name: 'DictationSettings' });

const props = defineProps<{
  voiceStatus: VoiceStatus | null;
  downloadPct: Record<string, number>;
}>();
const emit = defineEmits<{ reload: [] }>();

const { t } = useI18n();
const toast = useToast();
const voiceSettings = useVoiceSettingsStore();

const voiceModels = ref<VoiceModel[]>([]);
const accel = ref<AccelStatus | null>(null);
const inputDevices = ref<{ label: string; value: string }[]>([]);
const importingBinary = ref(false);
const asrResult = ref<string | null>(null);
const asrMicDenied = ref(false);

async function loadModels() {
  try {
    voiceModels.value = await ipc.listVoiceModels();
  } catch {
    voiceModels.value = [];
  }
}

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

async function onDownloadBinary() {
  try {
    await ipc.downloadVoiceBinary('whisper');
    notifyDownloaded();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('[settings] downloadBinary failed:', 'whisper', msg);
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: msg,
      life: 8000,
    });
  } finally {
    emit('reload');
  }
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
    emit('reload');
  }
}

async function onDownloadModel(m: VoiceModel) {
  try {
    await ipc.downloadVoiceModel(m.id);
    notifyDownloaded();
  } catch {
    notifyVoiceError();
  } finally {
    await loadModels();
    emit('reload');
  }
}

async function onDeleteModel(m: VoiceModel) {
  try {
    await ipc.deleteVoiceModel(m.id);
  } catch {
    // best-effort
  }
  await loadModels();
  emit('reload');
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

onMounted(async () => {
  await loadModels();
  await loadAccel();
  await refreshInputDevices();
  navigator.mediaDevices?.addEventListener?.('devicechange', refreshInputDevices);
});
</script>

<template>
  <section data-test="dictation-settings">
    <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mt-4 mb-2">
      {{ t('settings.voiceDictation') }}
    </h3>

    <!-- Whisper binary (compact line) -->
    <div
      class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
    >
      <span>
        <i
          :class="
            props.voiceStatus?.binaryInstalled
              ? 'pi pi-check-circle text-green-500'
              : 'pi pi-exclamation-circle text-amber-500'
          "
          class="mr-1"
        />
        {{
          props.voiceStatus?.binaryInstalled
            ? t('settings.voiceBinaryInstalled')
            : t('settings.voiceBinaryMissing')
        }}
      </span>
      <div class="flex items-center gap-2">
        <Button
          v-if="!props.voiceStatus?.binaryInstalled"
          :label="t('settings.voiceDownloadBinary')"
          icon="pi pi-download"
          size="small"
          @click="onDownloadBinary"
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

    <!-- ASR model selector -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceModelSelector') }}
      </h3>
      <div class="flex items-center gap-2">
        <div class="flex-1 min-w-0">
          <Select
            v-model="voiceSettings.asrModelId"
            :options="asrOptions"
            option-label="label"
            option-value="value"
            :placeholder="t('settings.voiceModelSelectorLabel')"
            class="w-full"
            :pt="{ label: { class: 'truncate' } }"
            :aria-label="t('settings.voiceModelSelector')"
          />
        </div>
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

    <!-- Model management (disclosure) -->
    <details class="mt-3 rounded border border-surface-200 dark:border-surface-700">
      <summary class="cursor-pointer p-3 text-sm font-medium">
        {{ t('settings.voiceManageModels') }}
      </summary>
      <ul
        class="divide-y divide-surface-200 dark:divide-surface-700 border-t border-surface-200 dark:border-surface-700"
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
            <span v-if="props.downloadPct[m.id] !== undefined" class="text-xs font-mono opacity-70">
              {{ `${props.downloadPct[m.id]}%` }}
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
    </details>

    <!-- Microphone (input device) selector -->
    <div class="mt-3">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.voiceInputDevice') }}
      </h3>
      <div class="flex items-center gap-2">
        <div class="flex-1 min-w-0">
          <Select
            v-model="voiceSettings.inputDeviceId"
            :options="inputDevices"
            option-label="label"
            option-value="value"
            :placeholder="t('settings.voiceInputDeviceDefault')"
            show-clear
            class="w-full"
            :pt="{ label: { class: 'truncate' } }"
            :aria-label="t('settings.voiceInputDevice')"
            :empty-message="t('settings.voiceInputDeviceEmpty')"
          />
        </div>
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
  </section>
</template>
